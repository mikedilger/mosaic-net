use mosaic_core::*;
use mosaic_net::*;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_secret_key = {
        let mut csprng = rand::rngs::OsRng;
        SecretKey::generate(&mut csprng)
    };
    println!("Client public key: {}", client_secret_key.public());

    let server_public_key =
        PublicKey::from_printable("mopub03ctpjer5jfkd49rxe4767hk9ij6f8sdtryjnnru1bpwxhcykk54o")?;

    let server_socket: SocketAddr = "127.0.0.1:8081".parse()?;

    let client_config = ClientConfig::new(
        server_public_key,
        server_socket,
        Some(client_secret_key.clone()),
    )?;

    let client = client_config.client(None).await?;

    let mut channel = client.new_channel().await?;

    let record = {
        let payload = b"hello";
        let tags = OwnedTagSet::new();
        let parts = RecordParts {
            kind: Kind::CHAT_MESSAGE,
            deterministic_nonce: None,
            timestamp: Timestamp::now()?,
            flags: RecordFlags::empty(),
            tag_set: &tags,
            payload: payload.as_slice(),
        };
        OwnedRecord::new(&client_secret_key, &parts)?
    };

    let message = Message::new_submission(&record)?;

    channel.send(message).await?;

    if let Some(message) = channel.recv().await? {
        match message.message_type() {
            MessageType::SubmissionResult => {
                eprintln!("Submission result: {:?}", message.submission_result_code().unwrap());
            },
            _ => {
                eprintln!("Unexpected response: {message:?}");
            }
        }
    }

    client.close(0, b"client is done").await;

    Ok(())
}
