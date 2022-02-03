use libc::{TCSAFLUSH, ioctl, winsize, TIOCGWINSZ, printf};
use termios::*;
use std::io::{stdin, stdout, Read, Write, Stdin, Stdout, Error};
use std::mem::size_of;
use std::os::unix::io::AsRawFd;
use std::{process, mem};

fn main() {
    let mut raw_terminal = RawTerminal::enable_raw_mode();
    let screensize = raw_terminal.get_terminal_size().unwrap();

    raw_terminal.screencols = screensize.0;
    raw_terminal.screenrows = screensize.1;

    loop {
        //raw_terminal.editor_refresh_screen();
        raw_terminal.editor_process_keypress();
    }
}

pub fn ctrl(c: char) -> u8 {
    (c as u8) & 31u8
}

pub struct RawTerminal {
    stdin: Stdin,
    stdout: Stdout,
    screencols: u16,
    screenrows: u16,
    preview_terminal: Termios,
}

impl RawTerminal {
    fn enable_raw_mode() -> RawTerminal {
        let stdin = stdin();
        let stdout = stdout();
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
            stdout,
            screencols: 0,
            screenrows: 0,
            preview_terminal,
        }
    }

    fn disable_raw_mode(&self) {
       tcsetattr(self.stdin.as_raw_fd(), TCSAFLUSH, &self.preview_terminal).unwrap();
    }

    fn editor_read_key(&mut self) -> Result<u8,Error> {
       let mut c = [0u8;1]; 
       self.stdin.read(&mut c)?;
       Ok(c[0])
    }

    fn editor_process_keypress(&mut self) {
        let c = self.editor_read_key().unwrap();
        if c == ctrl('q') {
            process::exit(0);
        }
        let buffer = [c;1];
        self.stdout.write(&buffer);
        self.stdout.flush();
    }

    fn editor_refresh_screen(&mut self) {
        self.stdout.write(b"\x1b[2J");
        self.stdout.write(b"\x1b[H");
        self.stdout.flush();
        self.editor_draw_rows();
    }

    fn editor_draw_rows(&mut self) {
        for i in 0..=self.screenrows {
            self.stdout.write(b"~\r\n");
            self.stdout.flush();
        }
    }

    fn get_terminal_size(&mut self) -> Option<(u16,u16)> {
        self.stdout.write(b"\x1b[999C\x1b[999B");
        self.stdout.flush();
        self.get_cursor_position();
        Some((24,24))
    }

    fn get_cursor_position(&mut self) -> Option<(u16,u16)> {
        self.stdout.write(b"\x1b[6n");
        self.stdout.flush();

        let mut buffer = [0u8;32];
        let mut i = 0usize;

        while i < buffer.len() - 1 {
            let mut c = [0u8;1];
            if self.stdin.read(&mut c).is_err() { break; };
            if c[0] == 82 { break; };
            buffer[i] = c[0];
            i += 1;
        } 
        
        buffer[i] = 0u8;

        print!("\r\n&buf[0]: {}\r\n", &buffer[0]);
        print!("\r\n&buf[1]: {}\r\n", &buffer[1]);
        print!("\r\n&buf[2]: {}\r\n", &buffer[2]);
        print!("\r\n&buf[3]: {}\r\n", &buffer[3]);
        print!("\r\n&buf[4]: {}\r\n", &buffer[4]);
        print!("\r\n&buf[5]: {}\r\n", &buffer[5]);
        print!("\r\n&buf[6]: {}\r\n", &buffer[6]);

        //if buffer[0] != b'\x1b' || buffer[1] != b'[' { return None };
        
        Some((0,0))
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.editor_refresh_screen();
        self.disable_raw_mode();
    }
}
