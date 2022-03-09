use std::io::{self, Read};

#[derive(Debug,PartialEq)]
pub enum EditorKey {
    Char(u8),
    Ctrl(u8),
    Function(u8),
    Escape,
    ArrowUp,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Delete,
    BackSpace,
    Null,
    Undefined,
}

#[derive(Debug)]
pub struct InputKeys<R> {
    input_source: R,
}

impl<R: Read> Iterator for InputKeys<R> {
    type Item = io::Result<EditorKey>;
    
    fn next(&mut self) -> Option<io::Result<EditorKey>> {
        let mut buffer = [0_u8;1];
        self.input_source.read(&mut buffer).unwrap();
        Some(self.parse_input(buffer[0]))
    }
}

impl<R: Read> InputKeys<R> {
    fn parse_input(&mut self, b: u8) -> io::Result<EditorKey> {
        match b {
            0x00 => Ok(EditorKey::Null),
            0x01..=0x07 => Ok(EditorKey::Ctrl(b + 64_u8)),
            0x08 => Ok(EditorKey::BackSpace),
            0x09..=0x1a => Ok(EditorKey::Ctrl(b + 64_u8)),
            0x1b => self.parse_escape_sequence(), // start escape sequence
            0x1c..=0x1f => Ok(EditorKey::Ctrl(b + 64_u8)),
            0x20..=0x7e => Ok(EditorKey::Char(b)), // ASCII code
            0x7f => Ok(EditorKey::Delete),
            _ => Ok(EditorKey::Undefined),
        }
    }

    fn parse_escape_sequence(&mut self) -> io::Result<EditorKey> {
        let mut buffer: [u8;1] = [0];
        self.input_source.read(&mut buffer)?;
        match buffer[0] {
            b'[' => self.parse_csi(),
            _ => Ok(EditorKey::Undefined),
        }
    }

    fn parse_csi(&mut self) -> io::Result<EditorKey> {
        let mut buffer: [u8;1] = [0];
        self.input_source.read(&mut buffer)?;
        match buffer[0] {
            b'A' => Ok(EditorKey::ArrowUp),
            b'B' => Ok(EditorKey::ArrowDown),
            b'C' => Ok(EditorKey::ArrowRight),
            b'D' => Ok(EditorKey::ArrowLeft),
            b'H' => Ok(EditorKey::Home),
            b'F' => Ok(EditorKey::End),
            b'O' | b'1' | b'2' => self.parse_function_key(),
            b'3' => Ok(EditorKey::Delete),
            b'5' => Ok(EditorKey::PageUp),
            b'6' => Ok(EditorKey::PageDown),
            _    => Ok(EditorKey::Undefined),
        }
    }

    fn parse_function_key(&mut self) -> io::Result<EditorKey> {
        let mut buffer: [u8;1] = [0];
        self.input_source.read(&mut buffer)?;
        match buffer[0] {
            b'P' => Ok(EditorKey::Function(1)),
            b'Q' => Ok(EditorKey::Function(2)),
            b'R' => Ok(EditorKey::Function(3)),
            b'S' => Ok(EditorKey::Function(4)),
            b'5' => Ok(EditorKey::Function(5)),
            b'7' => Ok(EditorKey::Function(6)),
            b'8' => Ok(EditorKey::Function(7)),
            b'9' => Ok(EditorKey::Function(8)),
            b'0' => Ok(EditorKey::Function(9)),
            b'1' => Ok(EditorKey::Function(10)),
            b'3' => Ok(EditorKey::Function(11)),
            b'4' => Ok(EditorKey::Function(12)),
            _    => Ok(EditorKey::Undefined),
        }
    }
}

pub trait ReadKey {
    fn keys(self) -> InputKeys<Self> where Self: Sized;
}

impl<R: Read> ReadKey for R {
    fn keys(self) -> InputKeys<Self> where Self: Sized{ 
        InputKeys { input_source: self }
    } 
}


pub fn ctrl(c: char) -> EditorKey {
    EditorKey::Char((c as u8) & 31_u8)
}
