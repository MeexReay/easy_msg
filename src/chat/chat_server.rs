use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::io::Error;
use std::time::Duration;

use super::chat_packet::*;

pub struct ChatServer {
    pub host: String,
    pub name: String,
    pub messages: Vec<Message>,
    pub conns: Vec<TcpStream>
}

pub fn send_packet_all(server: &mut ChatServer, packet: ChatPacket) -> Result<(), Error> {
    for stream in &mut server.conns {
        send_packet(stream, &packet)?;
    }
    Ok(())
}

static mut MEOW_TIME: u32 = 0;

fn meow() {
    unsafe {
        MEOW_TIME += 1;
        println!("meow {}", MEOW_TIME);
    }
}

impl ChatServer {
    pub fn new(host: String, name: String) -> Self {
        ChatServer {
            host: host,
            name: name,
            messages: Vec::new(),
            conns: Vec::new()
        }
    }

    pub fn send_message(&mut self, msg: Message) -> Result<(), Error> {
        println!("{} | {} > {}", msg.sent_at, msg.author, msg.text);

        let packet = ChatPacket::build(b'm', &|p| {
            p.buffer.write_string(&msg.text);
            p.buffer.write_string(&msg.author);
            p.buffer.write_i64(msg.sent_at.timestamp());
        });

        send_packet_all(self, packet)?;
        self.messages.push(msg);

        Ok(())
    }

    pub fn send_message_packet(&mut self, stream: &mut TcpStream, msg: &Message) -> Result<(), Error> {
        let packet = ChatPacket::build(b'm', &|p| {
            p.buffer.write_string(&msg.text);
            p.buffer.write_string(&msg.author);
            p.buffer.write_i64(msg.sent_at.timestamp());
        });

        send_packet(stream, &packet)?;

        // println!("send message packet");

        Ok(())
    }

    pub fn send_chat_name(&mut self, stream: &mut TcpStream) -> Result<(), Error> {
        let packet = ChatPacket::build(b'n', &|p| {
            p.buffer.write_string(&self.name);
        });

        send_packet(stream, &packet)?;

        // println!("send chat name");

        Ok(())
    }

    pub fn join_client(this: Arc<Mutex<ChatServer>>, stream: &mut TcpStream, name: String) -> Result<(), Error> {
        println!("connected lol {}", name.clone());

        // dbg!(&this.lock().unwrap().messages);

        this.lock().unwrap().send_chat_name(stream)?;

        let messages = &this.lock().unwrap().messages.clone();

        for m in messages {
            this.lock().unwrap().send_message_packet(stream, m)?;
        }

        loop {
            let mut packet = read_packet(stream)?;

            match packet.id {
                b's' => {
                    this.lock().unwrap().send_message(Message::at_now(packet.buffer.read_string()?, name.clone()))?;
                },
                _ => {
                    break;
                }
            }
        }

        close_stream(stream)?;
        Ok(())
    }

    pub fn accept_client(this: Arc<Mutex<ChatServer>>, stream: &mut TcpStream) -> Result<(), Error> {
        println!("stream got!!! lol {:?}", &stream);

        let mut packet: ChatPacket = read_packet(stream)?;

        println!("packet got!!! lol {:?}", &packet);

        match packet.id {
            b'j' => {
                Self::join_client(this, stream, packet.buffer.read_string()?)
            }, _ => {

                close_stream(stream)?;
                Ok(())
            }
        }
    }

    pub fn run(self) {
        let listener: TcpListener = TcpListener::bind(self.host.clone()).expect("bind error");

        println!("binded lol {}", self.host.clone());

        let arc_self = Arc::new(Mutex::new(self));

        loop {
            let (mut socket, _) = listener.accept().expect("accept error");
    
            let local_self = arc_self.clone();

            thread::spawn(move || {
                meow();

                let socket_clone = match socket.try_clone() {
                    Ok(i) => i,
                    Err(_) => { return },
                };

                meow();

                let mut ind: usize = 0;

                {
                    let mut se = local_self.lock().unwrap();
                    ind = se.conns.len();
                    se.conns.push(socket_clone);
                }
                
                meow();

                match Self::accept_client(local_self.clone(), &mut socket) {
                    Ok(_) => { println!("disconnected normally") },
                    Err(_) => { println!("trahnutiy disconnect XDDDDDDDDDDD))))))000)0)0)))") },
                };
                
                meow();

                local_self.lock().unwrap().conns.remove(ind);
                
                meow();
            });
        }
    }
}