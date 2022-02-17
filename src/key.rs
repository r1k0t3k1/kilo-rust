use std::io::{self, Read};

#[derive(PartialEq)]
pub enum EditorKey {
    Char(u8),
    Ctrl(u8),
    Escape,
    ArrowUp,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
}

pub struct InputKeys<R> {
    input_source: R,
}

impl<R: Read> Iterator for InputKeys<R> {
    type Item = io::Result<EditorKey>;
    
    fn next(&mut self) -> Option<io::Result<EditorKey>> {
        let mut buffer = [0_u8;1];
        loop {
            self.input_source.read(&mut buffer).unwrap();
            match buffer[0] {
               126 => return Some(Ok(EditorKey::Char(126))),
                _ => (),
            }
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
