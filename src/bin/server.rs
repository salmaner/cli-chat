use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

use cli_chat::IMessage;
use std::thread;

use std::collections::HashMap;

use std::time::Duration;

pub struct User {
    pub stream: RwLock<TcpStream>,
}

pub type ClientsMap = Arc<RwLock<HashMap<String, User>>>;

/// Рассылает сообщение всем, кроме отправителя.
/// Формат пакета: [u32 BE длина][bitcode(IMessage)]
fn broadcast(clients: &ClientsMap, msg: &IMessage, skip_from: &str) {
    let bytes = bitcode::encode(msg);
    let len = bytes.len() as u32;

    let map = clients.read().unwrap();

    for (name, user) in map.iter() {
        if name == skip_from {
            continue;
        }

        let mut stream = user.stream.write().unwrap();

        let _ = stream.write_all(&len.to_be_bytes());
        let _ = stream.write_all(&bytes);
    }
}

/// Функция, которая выполняется в отдельном потоке ОС для каждого клиента.
fn handle_client(stream: TcpStream, clients_map: ClientsMap) {
    let peer = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("[server-error] Клиент не определился: {}", e);
            return;
        }
    };

    println!("[server] connected: {}", peer);

    if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(2))) {
        eprintln!(
            "[server] не удалось установить write_timeout для {}: {}",
            peer, e
        );
        return;
    }

    let mut stream = stream;

    let mut my_name: Option<String> = None;

    let mut len_buf = [0u8; 4];

    loop {
        if let Err(e) = stream.read_exact(&mut len_buf) {
            eprintln!("[server] read len error from {}: {}", peer, e);
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        if let Err(e) = stream.read_exact(&mut buf) {
            eprintln!("[server] read payload error from {}: {}", peer, e);
            break;
        }

        let msg = match bitcode::decode::<IMessage>(&buf) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[server] decode error from {}: {}", peer, e);
                continue;
            }
        };

        match &msg {
            IMessage::Join { name } => {
                println!("[server] {} представился как {}", peer, name);

                let clone_for_map = match stream.try_clone() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[server] try_clone для map не удался ({}): {}", peer, e);
                        break;
                    }
                };

                let user = User {
                    stream: RwLock::new(clone_for_map),
                };

                let mut map = clients_map.write().unwrap();
                map.insert(name.clone(), user);
                println!(
                    "[server] в реестре {} клиентов, имена: {:?}",
                    map.len(),
                    map.keys().collect::<Vec<_>>()
                );

                my_name = Some(name.clone());
            }
            IMessage::Chat { from, body } => {
                println!("[{}] {}: {}", peer, from, body);
                broadcast(&clients_map, &msg, from);
            }
            IMessage::StopServer => {
                println!("Клиент {} запросил остановку сервера.", peer);
                std::process::exit(0);
            }
        }
    }

    if let Some(name) = &my_name {
        let mut map = clients_map.write().unwrap();
        map.remove(name);
        println!("[server] {} ({}) покинул чат", name, peer);
    }

    println!("[server] disconnected: {}", peer);
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:44370")?;
    println!("Сервер запущен и ждет подключений... на: 127.0.0.1:44370");

    let clients_map: ClientsMap = Arc::new(RwLock::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients_map = Arc::clone(&clients_map);

                thread::spawn(move || handle_client(stream, clients_map));
            }
            Err(e) => eprintln!("[server] accept error: {}", e),
        }
    }

    Ok(())
}
