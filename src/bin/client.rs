use std::io::{self, BufRead, Read, Write};
use std::net::TcpStream;
use std::thread;

use clap::Parser;
use cli_chat::{Args, IMessage};

fn main() -> std::io::Result<()> {
    let user_name = Args::parse();
    let uname = match user_name.uname.is_empty() {
        true => "Guest".to_string(),
        false => user_name.uname,
    };
    let ip_port = match user_name.ip.is_empty() {
        true => "127.0.0.1:44370".to_string(),
        false => user_name.ip,
    };

    println!("Для отправки сообщения в чат, введите текст и нажмите Enter");
    println!("\tP.S. Если нужно прервать работу чата введите\t:exit");

    let mut stream = TcpStream::connect(ip_port)?;
    println!(
        "Вы успешно подключились к серверу! Ваш логин в чате: {}",
        uname
    );

    {
        let join = IMessage::Join {
            name: uname.clone(),
        };
        let bytes = bitcode::encode(&join);
        let len = bytes.len() as u32;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;
    }

    let mut read_stream = stream.try_clone()?;

    thread::spawn(move || {
        let mut len_buf = [0u8; 4];

        loop {
            if let Err(e) = read_stream.read_exact(&mut len_buf) {
                eprintln!("[client] read len error: {}", e);
                break;
            }
            let len = u32::from_be_bytes(len_buf) as usize;

            let mut buf = vec![0u8; len];
            if let Err(e) = read_stream.read_exact(&mut buf) {
                eprintln!("[client] read payload error: {}", e);
                break;
            }

            match bitcode::decode::<IMessage>(&buf) {
                Ok(IMessage::Chat { from, body }) => println!("{}: {}", from, body),

                Ok(IMessage::Join { .. }) => {
                    eprintln!("[client] неожиданно получен Join от сервера — игнорируем");
                }

                Ok(IMessage::StopServer) => {
                    eprintln!("[client] сервер не должен слать StopServer — игнорируем");
                }
                Err(e) => eprintln!("[client] decode error: {}", e),
            }
        }
    });

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let msg = line?;

        match msg.trim() {
            ":exit" => {
                println!("Закончили переписку в чате...");
                break;
            }
            ":stop:" => {
                let stop = IMessage::StopServer;
                let bytes = bitcode::encode(&stop);
                let len = bytes.len() as u32;
                stream.write_all(&len.to_be_bytes())?;
                stream.write_all(&bytes)?;
                println!("Клиент запросил остановку сервера.");
                std::process::exit(0);
            }
            _ => {
                let message = IMessage::Chat {
                    from: uname.clone(),
                    body: msg.clone(),
                };
                let bytes = bitcode::encode(&message);
                let len = bytes.len() as u32;
                stream.write_all(&len.to_be_bytes())?;
                stream.write_all(&bytes)?;
            }
        }
    }

    println!("Пользователь {uname}, вы покинули чат!\n");
    Ok(())
}
