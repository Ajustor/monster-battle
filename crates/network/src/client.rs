use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

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
