// Объявления для TcpStream
use std::io::{self, BufRead, BufReader, Write}; // Write Нужен для работы метода write_all
use std::net::TcpStream;
use std::thread;

// библиотека clap, объявления для обработки параметра clietn'а
use clap::Parser;

// Добавляем обработку параметра программы (client)
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Имя пользователя в чате
    #[arg(short, long)]
    uname: String,
}

// Точка входа программы client
fn main() -> std::io::Result<()> {
    // обрабатываем параметр прогрммы client -> main()
    let user_name = Args::parse();
    let uname = match user_name.uname.is_empty() {
        true => "Guest".to_string(),
        false => user_name.uname,
    };

    println!("Для отправки сообщения в чат, введите текст и нажмите Enter");
    println!("\tP.S. Если нужно прервать работу чата введите\t:exit");

    // 1. Подключаемся к серверу по его ip:port
    // stream - главный поток, "труба" по которой будем отправлять сообщения серверу
    let mut stream = TcpStream::connect("127.0.0.1:44370")?;
    println!(
        "Вы успешно подключились к серверу! Ваш логин в чате: {}",
        uname
    );

    // второй поток read_stream будет слушать "трубу" в ожидании сообщений от сервера
    // не зависимо от главного потока stream.
    let read_stream = stream.try_clone()?;
    // созданный поток read_stream отправляем в независмое плавание от главного потока stream
    // предварительно обернув его в буфер. move передает владение от stream внутрь нового потока.
    thread::spawn(move || {
        let reader = BufReader::new(read_stream);
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("{}", msg),
                Err(e) => {
                    eprintln!("[client] read error: {}", e);
                    break;
                }
            }
        }
    });

    // обработка ввода от пользователя
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let msg = line?;
        //<<< -----------------------------------------------------
        match msg.trim() {
            ":exit" => {
                print!("Закончили переписку в чате... ");
                break;
            }
            _ => {
                //println!("{}: {:#?}", uname, msg); // для тестов
                //------------------------------------------------------->>>
                let fullmess = format!("{}: {}", uname, msg);
                // сначала отправляем свое сообщение серверу
                stream.write_all(fullmess.as_bytes())?;
                // после отправки основного сообщения отправляем перевод новой строки
                // сервер это воспримет как окончание сообщения.
                stream.write_all(b"\n")?;
                match msg.trim() {
                    ":stop:" => {
                        println!("Клиент запросил остановку сервера.");
                        std::process::exit(0); // Клиент полностью закрывается прямо сейчас
                    }
                    _ => {}
                }
            }
        }
    }

    println!("Пользователь {uname}, вы покинули чат!\n");
    Ok(())
}
