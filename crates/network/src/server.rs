use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::protocol::NetMessage;

/// Serveur de jeu qui écoute les connexions entrantes.
pub struct GameServer {
    port: u16,
    /// Le message reçu du client.
    pub received: Arc<Mutex<Option<NetMessage>>>,
    /// Le stream de la connexion acceptée.
    pub stream: Arc<Mutex<Option<TcpStream>>>,
}

impl GameServer {
    /// Crée un nouveau serveur sur le port donné.
    pub fn new(port: u16) -> Self {
        Self {
            port,
            received: Arc::new(Mutex::new(None)),
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Démarre l'écoute et attend une connexion entrante.
    /// Retourne une erreur si le bind échoue.
    pub async fn accept_one(&self) -> Result<(), anyhow::Error> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;

        let (socket, _addr) = listener.accept().await?;
        *self.stream.lock().await = Some(socket);

        Ok(())
    }

    /// Envoie un message au client connecté.
    pub async fn send(&self, msg: &NetMessage) -> Result<(), anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas de connexion active"))?;

        let data = msg.to_bytes().map_err(|e| anyhow::anyhow!("{}", e))?;
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Reçoit un message du client connecté.
    pub async fn recv(&self) -> Result<NetMessage, anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas de connexion active"))?;

        read_message(stream).await
    }

    /// Port d'écoute.
    pub fn port(&self) -> u16 {
        self.port
    }
}

/// Lit un message préfixé par sa longueur depuis un TcpStream.
pub(crate) async fn read_message(stream: &mut TcpStream) -> Result<NetMessage, anyhow::Error> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 10 * 1024 * 1024 {
        return Err(anyhow::anyhow!("Message trop gros : {} octets", len));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let json = String::from_utf8(buf)?;
    let msg = NetMessage::from_json(&json).map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(msg)
}
