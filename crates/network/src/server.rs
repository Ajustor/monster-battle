use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::protocol::NetMessage;

/// Lit un message préfixé par sa longueur depuis un TcpStream.
pub async fn read_message(stream: &mut TcpStream) -> Result<NetMessage, anyhow::Error> {
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

/// Envoie un message préfixé par sa longueur sur un TcpStream.
pub async fn write_message(stream: &mut TcpStream, msg: &NetMessage) -> Result<(), anyhow::Error> {
    let data = msg.to_bytes().map_err(|e| anyhow::anyhow!("{}", e))?;
    stream.write_all(&data).await?;
    stream.flush().await?;
    Ok(())
}
