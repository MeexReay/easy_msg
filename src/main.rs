mod chat;
use chat::*;

use std::env;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use std::cell::{RefCell};
use std::thread;
use std::pin::pin;


fn main() {
    let args: Vec<String> = Vec::from_iter(env::args());
    
    if args.len() > 1 {
        if args[1].as_str() == "server" {
            let mut server = ChatServer::new(
                args[3].clone(), 
                args[2].clone());
            server.messages.push(Message {
                author: "avtor".to_string(),
                text: "text".to_string(),
                sent_at: Utc::now()
            });
            server.run();
        } else if args[1].as_str() == "client" {
            let console = Arc::new(Mutex::new(Console::new()));

            console.lock().unwrap().update();

            let mut client = ChatClient::new(args[2].clone(), console.clone());

            let console_copy = console.clone();

            client.connect(args[3].clone()).unwrap();

            let mut input_stream = client.stream.as_mut().unwrap().try_clone().unwrap();
            thread::spawn(move || {
                Console::run_input_loop(
                    console_copy, 
                    ChatClient::on_enter, 
                    &mut input_stream);
            });

            client.communicate().unwrap();
        }
    }

    // let моя_коза = МояГоворящаяКоза::создать(true);
    // моя_коза.финты_ушами();
    // моя_коза.поздароваться();
    // моя_коза.привет("Товарищ Сталин".to_string());
}

// struct МояГоворящаяКоза {
//     здоровье: u128,
//     еда_в_холодильнике: u8,
//     возраст: u128,
//     коммунист: bool,
//     кличка: String
// }

// impl МояГоворящаяКоза {
//     fn создать(коммунист: bool) -> МояГоворящаяКоза {
//         МояГоворящаяКоза {
//             здоровье: 9999999999999999999,
//             еда_в_холодильнике: 0,
//             возраст: 9,
//             коммунист: коммунист,
//             кличка: "Ленин".to_string()
//         }
//     }

//     fn финты_ушами(&self) {
//         println!("{} крутит финты ушами", self.кличка);
//     }

//     fn поздароваться(&self) {
//         println!("{} поздаровался", self.кличка);
//     }
// }


// trait ГоворящееЧтото {
//     fn моя_кличка(&self) -> &String;

//     fn привет(&self, имя: String) {
//         println!("{} сказал привет {}", self.моя_кличка(), имя);
//     }
// }

// impl ГоворящееЧтото for МояГоворящаяКоза {
//     fn моя_кличка(&self) -> &String {
//         &self.кличка
//     }
// }

// console.run_event_loop():
//   on_enter:
//     client.send_message()

// client.connect():
//   on_message:
//     console.print()