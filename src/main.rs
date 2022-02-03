use libc::TCSAFLUSH;
use termios::*;
use std::io::{stdin, stdout, Read, Write, Stdin, Stdout, Error};
use std::os::unix::io::AsRawFd;
use std::process;

fn main() {
    let mut raw_terminal = RawTerminal::enable_raw_mode();

    //loop {
    //  let mut c: [u8;1] = [0];
    //  if raw_terminal.stdin.read(&mut c).is_ok() {
    //      if (c[0] <= 31) || (c[0] == 127) {
    //          print!("{}(control)\r\n", c[0])
    //      } else{
    //          print!("{}\r\n",c[0]);
    //      }
    //  } else {
    //      break;
    //  }
        // Press Ctrl-Q to quit
    //  if c[0] == ctrl('q') {
    //      break;
    //  }
    //}
    loop {
        raw_terminal.editor_refresh_screen();
        raw_terminal.editor_process_keypress();
    }
    //return;
}

pub fn ctrl(c: char) -> u8 {
    (c as u8) & 31u8
}

pub struct RawTerminal {
    stdin: Stdin,
    stdout: Stdout,
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
    }

    fn editor_refresh_screen(&mut self) {
        self.stdout.write(b"\x1b[2J");
        self.stdout.write(b"\x1b[H");
        self.stdout.flush();

        self.editor_draw_rows();
    }

    fn editor_draw_rows(&mut self) {
        for i in 0..23 {
            self.stdout.write(b"~\r\n");
            self.stdout.flush();
        }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.editor_refresh_screen();
        self.disable_raw_mode();
    }
}
