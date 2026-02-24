use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::protocol::NetMessage;
use crate::server::{read_message, write_message};

/// Client réseau qui se connecte au serveur relais centralisé.
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

    /// Se connecte au serveur relais.
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

        write_message(stream, msg).await
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
        self.stream
            .try_lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
}
