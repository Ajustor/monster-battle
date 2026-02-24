use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use crate::protocol::NetMessage;

/// Lit un message NetMessage depuis un WebSocket.
pub async fn read_message<S>(
    ws: &mut WebSocketStream<S>,
) -> Result<NetMessage, anyhow::Error>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    loop {
        match ws.next().await {
            Some(Ok(Message::Text(text))) => {
                let msg: NetMessage = serde_json::from_str(&text)?;
                return Ok(msg);
            }
            Some(Ok(Message::Close(_))) | None => {
                return Err(anyhow::anyhow!("Connexion fermée"));
            }
            Some(Ok(_)) => continue, // ignore ping/pong/binary frames
            Some(Err(e)) => return Err(e.into()),
        }
    }
}

/// Envoie un message NetMessage sur un WebSocket.
pub async fn write_message<S>(
    ws: &mut WebSocketStream<S>,
    msg: &NetMessage,
) -> Result<(), anyhow::Error>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let json = serde_json::to_string(msg)?;
    ws.send(Message::Text(json.into())).await?;
    Ok(())
}
