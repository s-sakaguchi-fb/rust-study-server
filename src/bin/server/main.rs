use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TCPリスナーをポート1234でバインドします
    let listener = TcpListener::bind("0.0.0.0:1234").await?;
    
    // ブロードキャストチャネルを作成します
    let (tx, _rx) = broadcast::channel::<String>(100);
    
    println!("Server running on port 1234...");

    loop {
        // 新しいクライアント接続を待ちます
        let (socket, _) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // クライアントを処理するタスクをスパーンします
        tokio::spawn(async move {
            handle_client(socket, tx, rx).await;
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) {
    let peer = socket.peer_addr().unwrap();
    println!("New connection: {}", peer);

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            // クライアントからのメッセージを読み込みます
            result = reader.read_line(&mut line) => {
                println!("Client receive: {}", peer);
                if result.unwrap_or(0) == 0 {
                    // 接続が閉じられた場合、処理を終了します
                    println!("Connection closed.");
                    break;
                }

                let msg = format!("{}: {}", peer, line);
                // メッセージを全クライアントに送信します
                if tx.send(msg.clone()).is_err() {
                    println!("Broadcast send error.");
                    break;
                }

                println!("Broadcast sent: '{}'", msg.trim());

                line.clear();
            }

            // ブロードキャストメッセージをクライアントに送信します
            result = rx.recv() => {
                if let Ok(msg) = result {
                    println!("Broadcast receive: {}: '{}'", peer, msg.trim());
                    if writer.write_all(msg.as_bytes()).await.is_err() {
                        break;
                    }
                    println!("Client sent: {}: '{}'", peer, msg.trim());
                }
            }
        }
    }

    println!("Connection closed: {}", peer);
}
