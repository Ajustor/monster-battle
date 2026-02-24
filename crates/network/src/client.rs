use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::protocol::NetMessage;
use crate::server::read_message;

/// Client réseau qui se connecte à un serveur distant.
pub struct GameClient {
    /// Le stream de la connexion.
    pub stream: Arc<Mutex<Option<TcpStream>>>,
}

impl GameClient {
    /// Crée un nouveau client non connecté.
    pub fn new() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Se connecte à un serveur distant.
    pub async fn connect(&self, addr: &str) -> Result<(), anyhow::Error> {
        let socket = TcpStream::connect(addr).await?;
        *self.stream.lock().await = Some(socket);
        Ok(())
    }

    /// Envoie un message au serveur.
    pub async fn send(&self, msg: &NetMessage) -> Result<(), anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas connecté"))?;

        let data = msg.to_bytes().map_err(|e| anyhow::anyhow!("{}", e))?;
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Reçoit un message du serveur.
    pub async fn recv(&self) -> Result<NetMessage, anyhow::Error> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pas connecté"))?;

        read_message(stream).await
    }

    /// Teste si le client est connecté.
    pub fn is_connected(&self) -> bool {
        // On ne peut pas vérifier sans await, donc on vérifie si le stream est Some
        // via try_lock
        self.stream
            .try_lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
}
