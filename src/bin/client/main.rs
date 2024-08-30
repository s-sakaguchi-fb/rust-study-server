use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // サーバに接続
    let stream = TcpStream::connect("127.0.0.1:1234").await?;
    let peer = stream.local_addr().unwrap();
    let (reader, mut writer) = stream.into_split();

    println!("I am {}", peer);

    // 標準入力を読み取るためのリーダーを準備
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut line = String::new();

    // サーバからのメッセージを受信するためのタスクをスパーン
    let (tx, mut rx) = mpsc::channel::<String>(100);
    tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // サーバからのメッセージを読み取るループ
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("Server socket closed");
                    let _ = tx.send("".to_string()).await;
                    break; // 接続が閉じられた場合
                }
                Ok(_) => {
                    //println!("Received from server: {}", line);
                    if tx.send(line.clone()).await.is_err() {
                        break;
                    }
                }
                Err(_) => {
                    println!("Error occured");
                    break;
                }
            }
        }
    });

    println!("Connected to the server. Type a message and press Enter to send:");

    // 標準入力とサーバからの受信を並行して処理
    loop {
        tokio::select! {
            // 標準入力からのメッセージ読み込み
            result = stdin_reader.read_line(&mut line) => {
                if result.unwrap_or(0) == 0 {
                    // 標準入力が終了した場合、処理を終了
                    println!("Exitting.");
                    break;
                }

                // メッセージをサーバに送信
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                line.clear();
            }
            // サーバからのメッセージを受信して表示
            Some(msg) = rx.recv() => {
                if msg.is_empty() {  // サーバから切断された
                    println!("Press enter to exit.");
                    break;
                }
                print!("{}", msg);
            }
        }
    }

    Ok(())
}
