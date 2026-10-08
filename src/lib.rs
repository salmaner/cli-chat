use bitcode::{Decode, Encode};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cli-chat", about = "Простой CLI-чат")]
pub struct Args {
    /// Имя пользователя в чате
    #[arg(short, long)]
    pub uname: String,
    /// IP-адрес сервера
    #[arg(short, long)]
    pub ip: String,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum IMessage {

    Join {
        name: String,
    },

    /// Обычное сообщение в чат: кто и что написал — в одном значении.
    /// Сервер рассылает его всем, КРОМЕ отправителя (отправитель видит
    /// свой текст через эхо терминала).
    Chat {
        from: String,
        body: String,
    },

    StopServer,
}
