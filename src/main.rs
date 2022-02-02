use libc::{iscntrl, STDIN_FILENO, TCSAFLUSH};
use termios::*;
use std::io::{stdin, Read, Stdin};
use std::os::unix::io::AsRawFd;

fn main() {
    let mut raw_terminal = RawTerminal::enable_raw_mode();
    let mut c: [u8;1] = [0];

    loop {
        if raw_terminal.stdin.read(&mut c).is_ok() && c[0] != 113{
            if (c[0] <= 31) || (c[0] == 127) {
                print!("{}(control)\r\n", c[0])
            } else{
                print!("{}\r\n",c[0]);
            }
        } else {
            break;
        }
    }
    return;
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

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios).unwrap();
        RawTerminal {
            stdin,
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
