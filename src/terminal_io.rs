use std::io::{self,Write};
use termios::*;

use crate::sys::unix;

pub struct RawTerminal<W: Write> {
    pub output: W,
    pub preview_terminal: Termios,
}

impl<W:Write> RawTerminal<W> {
    pub fn suspend_raw_mode(&mut self) -> io::Result<()> {
       self.output.write(b"\x1b[2J".to_vec().as_mut()).unwrap();
       self.output.write(b"\x1b[0G\x1b[0d".to_vec().as_mut()).unwrap();
       unix::set_terminal_setting(&self.preview_terminal)
    }

    pub fn resume_raw_mode(&mut self) -> io::Result<()> {
        let raw_terminal = unix::get_raw_terminal_setting()?;
        unix::set_terminal_setting(&raw_terminal)
    }
}

pub trait EnableRawMode: Write + Sized {
    fn enable_raw_mode(self) -> io::Result<RawTerminal<Self>>;
}

impl<W: Write> EnableRawMode for W {
    fn enable_raw_mode(self) -> io::Result<RawTerminal<W>> {
        let prev = unix::get_terminal_setting()?;
        unix::get_raw_terminal_setting()?;
        Ok(RawTerminal {
            output: self,
            preview_terminal: prev,
        })
    }
}

