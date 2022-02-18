use std::io::{self, Read};

#[derive(Debug,PartialEq)]
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

#[derive(Debug)]
pub struct InputKeys<R> {
    pub input_source: R,
}

impl<R: Read> Iterator for InputKeys<R> {
    type Item = io::Result<EditorKey>;
    
    fn next(&mut self) -> Option<io::Result<EditorKey>> {
        let mut buffer = [0_u8;4];
        loop {
            self.input_source.read(&mut buffer).unwrap();
            match buffer[0] {
               17  => return Some(Ok(EditorKey::Ctrl(113))),
               27 => match buffer[1] {
                   91 => match buffer[2] {
                      65 => return Some(Ok(EditorKey::ArrowUp)),
                      66 => return Some(Ok(EditorKey::ArrowDown)),
                      67 => return Some(Ok(EditorKey::ArrowRight)),
                      68 => return Some(Ok(EditorKey::ArrowLeft)),
                      70 => return Some(Ok(EditorKey::End)),
                      72 => return Some(Ok(EditorKey::Home)),
                      _  => (),
                   }
                   _ => (),
               }
                _ => return Some(Ok(EditorKey::Char(1))),
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
