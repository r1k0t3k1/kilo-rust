use termios::*;
use std::io::{ Stdin, Stdout, stdin, stdout };
use std::os::unix::io::AsRawFd;

pub struct RawTerminal {
    preview_terminal: Termios,
}

impl RawTerminal {
    fn enable_raw_mode() -> RawTerminal {
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
        RawTerminal { preview_terminal }
    }


    fn disable_raw_mode(&self) {
       tcsetattr(self.stdin.as_raw_fd(), TCSANOW, &self.preview_terminal).unwrap();
    }
}
