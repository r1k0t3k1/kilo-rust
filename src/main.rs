use libc::{STDIN_FILENO, TCSAFLUSH};
use nix::unistd;
use termios::*;

fn main() {
    let raw_terminal = RawTerminal::enable_raw_mode();
    let mut c: [u8;1] = [0];
    while unistd::read(STDIN_FILENO, &mut c).is_ok() 
        && c[0] != 113{
        
    }
    return;
}

pub struct RawTerminal {
    preview_terminal: Termios,
}

impl RawTerminal {
    fn enable_raw_mode() -> RawTerminal {
        let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
        let preview_terminal = termios;
        termios.c_lflag &= !(ECHO);
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios).unwrap();
        RawTerminal {
            preview_terminal,
        }
    }

    fn disable_raw_mode(&self) {
       tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.preview_terminal).unwrap();
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.disable_raw_mode();
    }
}
