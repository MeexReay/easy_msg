use std::cell::RefCell;
use std::io::{Error, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use chrono::prelude::*;

use super::chat_packet::*;

#[derive(Debug)]
pub struct Console  {
    pub input: String,
    pub lines: Vec<String>
}

impl Console {
    pub fn new() -> Console {
        Console {
            input: String::new(),
            lines: Vec::new()
        }
    }

    pub fn meow(&mut self) {
        self.lines.push("meow".to_string());
        self.update();
    }

    pub fn add_line(&mut self, text: String) {
        self.lines.push(text);
        self.update();
    }

    pub fn run_input_loop(this: Arc<Mutex<Console>>, on_enter: fn(&mut TcpStream, String) -> (), client: &mut TcpStream) -> Result<(), Error> {
        let ge = getch::Getch::new();
        loop {
            match ge.getch() {
                Ok(i) => { 
                    let mut th = this.lock().unwrap();

                    match i {
                        127 => {
                            if th.input.len() > 0 {
                                th.input = (&th.input[..th.input.len()-1]).to_string();
                            }
                            th.update();
                        }, 10 => {
                            // th.meow();
                            on_enter(client, th.input.clone());
                            th.input = String::new();
                            th.update();
                        }, _ => {
                            let ch = &String::from_utf8(vec![i]).unwrap();
                            // let was = th.input.clone();
                            th.input.push_str(ch);
                            th.update();
                        }
                    }
                },
                Err(_) => {},
            }
        };
    }

    pub fn update(&mut self) {
        clearscreen::clear().unwrap();

        let mut text = "\n".to_string();

        for l in self.lines.as_slice() {
            text += &(l.to_string() + "\n");
        }

        text += &("> ".to_string() + self.input.as_str());

        std::io::stdout()
            .write_all(text.as_bytes())
            .unwrap();

        std::io::stdout()
            .flush()
            .unwrap();
    }
}

#[derive(Debug)]
pub struct ChatClient {
    pub me: String,
    pub messages: Vec<Message>,
    pub chat_name: String,
    pub stream: Option<TcpStream>,
    pub console: Arc<Mutex<Console>>
}

impl ChatClient {
    pub fn new(name: String, console: Arc<Mutex<Console>>) -> Self {
        ChatClient {
            me: name, 
            messages: Vec::new(), 
            chat_name: String::from("Chat"), 
            stream: None,
            console: console
        }
    }

    pub fn on_enter(stream: &mut TcpStream, text: String) {
        Self::send_message(stream, text).unwrap();
    }

    pub fn on_message(&mut self, msg: &Message) {
        self.console.lock().unwrap().add_line(format!("{} | {} > {}", msg.sent_at, msg.author, msg.text));
    }

    pub fn communicate(&mut self) -> Result<(), Error> {
        Self::send_join(&mut self.stream.as_mut().unwrap(), &self.me)?;

        loop {
            let mut packet = read_packet(&mut self.stream.as_mut().unwrap())?;

            // dbg!(&packet);

            match packet.id {
                b'm' => {
                    let msg = Message::new( 
                        packet.buffer.read_string()?, 
                        packet.buffer.read_string()?, 
                        DateTime::from_timestamp(packet.buffer.read_i64()?,0).unwrap()
                    );

                    self.on_message(&msg);

                    self.messages.push(msg);
                }, b'n' => {
                    self.chat_name = packet.buffer.read_string()?;
                }, _ => {
                    println!("packet error");
                    break;
                }
            }
        }

        close_stream(&mut self.stream.as_mut().unwrap())?;
        Ok(())
    }

    pub fn connect(&mut self, host: String) -> Result<(), Error> {
        self.stream = Some(TcpStream::connect(host.clone())?);

        Ok(())
    }

    pub fn send_message(stream: &mut TcpStream, text: String) -> Result<(), Error> {
        let packet = ChatPacket::build(b's', &|p| {
            p.buffer.write_string(&text);
        });

        send_packet(stream, &packet)
    }

    pub fn send_join(stream: &mut TcpStream, name: &String) -> Result<(), Error> {
        let packet = ChatPacket::build(b'j', &|p| {
            p.buffer.write_string(&name);
        });

        send_packet(stream, &packet)
    }

    pub fn send_quit(stream: &mut TcpStream) -> Result<(), Error> {
        let packet = ChatPacket::empty(b'q');

        send_packet(stream, &packet)
    }
}