use std::net::IpAddr;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, client_async_tls_with_config, connect_async,
};

use crate::protocol::NetMessage;
use crate::server::{read_message, write_message};

/// Installe le CryptoProvider rustls (ring) une seule fois.
fn ensure_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Type concret du WebSocket côté client (peut être TLS ou non).
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Client réseau qui se connecte au serveur relais via WebSocket.
pub struct GameClient {
    /// Le stream WebSocket de la connexion.
    pub stream: Arc<Mutex<Option<WsStream>>>,
}

impl GameClient {
    /// Crée un nouveau client non connecté.
    pub fn new() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Se connecte au serveur relais via WebSocket.
    /// Accepte une adresse simple ("host", "host:port") ou une URL complète ("ws://…", "wss://…").
    pub async fn connect(&self, addr: &str) -> Result<(), anyhow::Error> {
        ensure_crypto_provider();
        let url = make_ws_url(addr);
        let (ws_stream, _) = connect_async(&url).await?;
        *self.stream.lock().await = Some(ws_stream);
        Ok(())
    }

    /// Se connecte via une IP déjà résolue (contourne `getaddrinfo`).
    /// `addr` est le nom d'hôte original (utilisé pour TLS SNI et l'URL WebSocket).
    /// `ip` est l'adresse IP résolue par l'appelant.
    pub async fn connect_with_resolved_ip(
        &self,
        addr: &str,
        ip: IpAddr,
    ) -> Result<(), anyhow::Error> {
        ensure_crypto_provider();
        let url = make_ws_url(addr);
        let port = ws_port(&url);
        let tcp_stream = TcpStream::connect((ip, port)).await?;
        let (ws_stream, _) = client_async_tls_with_config(&url, tcp_stream, None, None).await?;
        *self.stream.lock().await = Some(ws_stream);
        Ok(())
    }

    /// Envoie un message au serveur.
    pub async fn send(&self, msg: &NetMessage) -> Result<(), anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let ws = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas connecté"))?;

        write_message(ws, msg).await
    }

    /// Reçoit un message du serveur.
    pub async fn recv(&self) -> Result<NetMessage, anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let ws = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas connecté"))?;

        read_message(ws).await
    }

    /// Teste si le client est connecté.
    pub fn is_connected(&self) -> bool {
        self.stream
            .try_lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
}

/// Vérifie si le serveur est joignable en tentant une connexion WebSocket.
pub async fn check_server_health(addr: &str) -> bool {
    ensure_crypto_provider();
    let url = make_ws_url(addr);
    match connect_async(&url).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Vérifie si le serveur est joignable via une IP déjà résolue.
pub async fn check_server_health_resolved(addr: &str, ip: IpAddr) -> bool {
    ensure_crypto_provider();
    let url = make_ws_url(addr);
    let port = ws_port(&url);
    let tcp_stream = match TcpStream::connect((ip, port)).await {
        Ok(s) => s,
        Err(_) => return false,
    };
    client_async_tls_with_config(&url, tcp_stream, None, None)
        .await
        .is_ok()
}

/// Récupère la version du serveur via le protocole WebSocket.
/// Retourne `Some("x.y.z")` en cas de succès, `None` si le serveur est injoignable.
pub async fn check_server_version(addr: &str) -> Option<String> {
    ensure_crypto_provider();
    let url = make_ws_url(addr);
    let (mut ws_stream, _) = connect_async(&url).await.ok()?;

    write_message(&mut ws_stream, &NetMessage::VersionCheck)
        .await
        .ok()?;

    let msg = read_message(&mut ws_stream).await.ok()?;
    match msg {
        NetMessage::VersionInfo { version } => Some(version),
        _ => None,
    }
}

/// Récupère la version du serveur via une IP déjà résolue.
pub async fn check_server_version_resolved(addr: &str, ip: IpAddr) -> Option<String> {
    ensure_crypto_provider();
    let url = make_ws_url(addr);
    let port = ws_port(&url);
    let tcp_stream = TcpStream::connect((ip, port)).await.ok()?;
    let (mut ws_stream, _) = client_async_tls_with_config(&url, tcp_stream, None, None)
        .await
        .ok()?;

    write_message(&mut ws_stream, &NetMessage::VersionCheck)
        .await
        .ok()?;

    let msg = read_message(&mut ws_stream).await.ok()?;
    match msg {
        NetMessage::VersionInfo { version } => Some(version),
        _ => None,
    }
}

/// Construit l'URL WebSocket à partir d'une adresse serveur.
/// - "host"         → "wss://host/ws"   (via Cloudflare / TLS)
/// - "host:port"    → "ws://host:port/ws" (LAN / local)
/// - "ws://…"       → inchangé
fn make_ws_url(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else if addr.contains(':') {
        format!("ws://{}/ws", addr)
    } else {
        format!("wss://{}/ws", addr)
    }
}

/// Extrait le port depuis une URL WebSocket (`wss://` → 443, `ws://` → 80).
fn ws_port(url: &str) -> u16 {
    // Si l'URL contient un port explicite dans la partie host, l'utiliser
    if let Some(rest) = url
        .strip_prefix("wss://")
        .or_else(|| url.strip_prefix("ws://"))
    {
        let host_part = rest.split('/').next().unwrap_or(rest);
        if let Some(port_str) = host_part.rsplit(':').next() {
            if let Ok(port) = port_str.parse::<u16>() {
                return port;
            }
        }
    }
    if url.starts_with("wss://") { 443 } else { 80 }
}
