use std::io::{Read, Write, Error};
use std::net::{TcpStream, Shutdown};

use bytebuffer::ByteBuffer;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct Message {
    pub author: String,
    pub text: String,
    pub sent_at: DateTime<Utc>
}

impl Message {
    pub fn new(text: String, 
                author: String, 
                sent_at: DateTime<Utc>) -> Message {
        Message {
            author,
            text,
            sent_at
        }
    }

    pub fn at_now(text: String, author: String) -> Message {
        Message {
            author,
            text,
            sent_at: Utc::now()
        }
    }
}

pub fn close_stream(stream: &mut TcpStream) -> Result<(), Error> {
    stream.shutdown(Shutdown::Both)
}

#[derive(Debug)]
pub struct ChatPacket {
    pub id: u8,
    pub buffer: ByteBuffer
}

impl ChatPacket {
    pub fn new(id: u8, buffer: ByteBuffer) -> ChatPacket {
        ChatPacket { id, buffer }
    }

    pub fn empty(id: u8) -> ChatPacket {
        ChatPacket { id, buffer: ByteBuffer::new() }
    }

    pub fn from_bytes(id: u8, bytes: &[u8]) -> ChatPacket {
        ChatPacket { id, buffer: ByteBuffer::from_bytes(bytes) }
    }

    pub fn build(id: u8, builder: &dyn Fn(&mut ChatPacket) -> ()) -> ChatPacket {
        let mut packet = Self::empty(id);
        builder(&mut packet);
        packet
    }
}

pub fn read_packet(stream: &mut TcpStream) -> Result<ChatPacket, Error> {
    let length = stream.read_u64::<BigEndian>()? as usize;

    let packet_id = stream.read_u8()?;
    let mut data: Vec<u8> = vec![0; length - 1];
    stream.read(&mut data)?;

    Ok(ChatPacket::from_bytes(packet_id, &data))
}

pub fn send_packet(stream: &mut TcpStream, packet: &ChatPacket) -> Result<(), Error> {
    let mut buf = ByteBuffer::new();

    buf.write_u8(packet.id);
    buf.write(packet.buffer.as_bytes())?;

    stream.write_u64::<BigEndian>(buf.len() as u64)?;
    stream.write(&buf.as_bytes())?;

    Ok(())
}