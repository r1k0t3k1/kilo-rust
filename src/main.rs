use libc::TCSAFLUSH;
use termios::*;
use std::io::{stdin, Read, Stdin};
use std::os::unix::io::AsRawFd;

fn main() {
    let mut raw_terminal = RawTerminal::enable_raw_mode();

    loop {
        let mut c: [u8;1] = [0];
        if raw_terminal.stdin.read(&mut c).is_ok() && c[0] != 113{
            if (c[0] <= 31) || (c[0] == 127) {
                print!("{}(control)\r\n", c[0])
            } else{
                print!("{}\r\n",c[0]);
            }
        } else {
            break;
        }
        // Press Ctrl-Q to quit
        if c[0] == ctrl('q') {
            break;
        }
    }
    return;
}

pub fn ctrl(c: char) -> u8 {
    (c as u8) & 31u8
}

pub struct RawTerminal {
    stdin: Stdin,
    preview_terminal: Termios,
}

impl RawTerminal {
    fn enable_raw_mode() -> RawTerminal {
        let stdin = stdin();
        let mut termios = Termios::from_fd(stdin.as_raw_fd()).unwrap();
        let preview_terminal = termios;
        // echo off
        termios.c_lflag &= !(ECHO);
        // turn off canonical mode
        termios.c_lflag &= !(ICANON);
        // turn off Ctrl-C and Ctrl-Z signals
        termios.c_lflag &= !(ISIG);
        // disable Ctrl-S and Ctrl-Q
        termios.c_iflag &= !(IXON);
        // disable Ctrl-V
        termios.c_lflag &= !(IEXTEN);
        // fix Ctrl-M
        termios.c_iflag &= !(ICRNL);
        // turn off all output processing
        termios.c_oflag &= !(OPOST);
        // miscellaneous flags
        termios.c_iflag &= !(BRKINT);
        termios.c_iflag &= !(INPCK);
        termios.c_iflag &= !(ISTRIP);
        termios.c_cflag |= CS8;
        // timeout for read
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        tcsetattr(stdin.as_raw_fd(), TCSAFLUSH, &termios).unwrap();
        RawTerminal {
            stdin,
            preview_terminal,
        }
    }

    fn disable_raw_mode(&self) {
       tcsetattr(self.stdin.as_raw_fd(), TCSAFLUSH, &self.preview_terminal).unwrap();
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.disable_raw_mode();
    }
}
