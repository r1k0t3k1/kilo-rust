use libc::TCSAFLUSH;
use termios::*;
use std::io::{stdin, stdout, Read, Write, Stdin, Stdout, Error };
use std::os::unix::io::AsRawFd;
use std::{process, char, str, u8 };

const VERSION: &str = "0.0.1";

#[repr(u8)]
#[derive(PartialEq)]
pub enum EditorKey {
    Char(u8),
    Escape,
    ArrowUp,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
}

fn main() {
    let mut raw_terminal = RawTerminal::enable_raw_mode();
    let screensize = raw_terminal.get_terminal_size().unwrap();

    raw_terminal.screenrows = screensize.0;
    raw_terminal.screencols = screensize.1;

    loop {
        raw_terminal.editor_refresh_screen();
        raw_terminal.editor_process_keypress();
    }
}

pub fn ctrl(c: char) -> EditorKey {
    EditorKey::Char((c as u8) & 31_u8)
}

pub struct RawTerminal {
    stdin: Stdin,
    stdout: Stdout,
    append_buffer: Vec<u8>,
    screencols: u16,
    screenrows: u16,
    cursor_x: u16,
    cursor_y: u16,
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
    
        let buf: Vec<u8> = vec![];

        tcsetattr(stdin.as_raw_fd(), TCSAFLUSH, &termios).unwrap();
        RawTerminal {
            stdin,
            stdout,
            append_buffer: buf,
            screencols: 0,
            screenrows: 0,
            cursor_x: 0,
            cursor_y: 0,
            preview_terminal,
        }
    }


    fn disable_raw_mode(&self) {
       tcsetattr(self.stdin.as_raw_fd(), TCSAFLUSH, &self.preview_terminal).unwrap();
    }

    fn editor_read_key(&mut self) -> Result<EditorKey,Error> {
       let mut c = [0_u8;1]; 
       self.stdin.read(&mut c)?;
       if c[0] == 27 {
            let mut esc = [0_u8;3];
            self.stdin.read(&mut esc)?;
            if esc[0] == 91 {
                match esc[1] {
                    65 => return Ok(EditorKey::ArrowUp),
                    66 => return Ok(EditorKey::ArrowDown),
                    67 => return Ok(EditorKey::ArrowRight),
                    68 => return Ok(EditorKey::ArrowLeft),
                    _  => (),
                }
            }
            return Ok(EditorKey::Escape);
       } 
       Ok(EditorKey::Char(c[0]))
    }

    fn editor_process_keypress(&mut self) {
        let c = self.editor_read_key().unwrap();
        if c == ctrl('q') {
            process::exit(0);
        }
        self.editor_move_cursor(&c);
        match c {
            EditorKey::Char(ch) => {
                let byte = [ch;1];
                self.stdout.write(&byte).unwrap();}
            _ => (),
        }
    }

    fn editor_refresh_screen(&mut self) {
        self.append_buffer.append(b"\x1b[?25l\x1b[H".to_vec().as_mut());
        self.editor_draw_rows();
        self.append_buffer.append(format!("\x1b[{};{}H",self.cursor_y + 1, self.cursor_x + 1)
            .as_bytes()
            .to_vec()
            .as_mut());
        self.append_buffer.append(b"\x1b[?25h".to_vec().as_mut());
        self.stdout.write_all(self.append_buffer.as_slice()).unwrap();
        self.stdout.flush().unwrap();
    }

    fn editor_draw_rows(&mut self) {
        for i in 0..self.screenrows {
            self.append_buffer.append(b"~\x1b[K".to_vec().as_mut());

            if i == self.screenrows / 3 {
                let message = format!("riko editor -- version {}", VERSION); 

                let padding_count = (self.screencols - message.len() as u16) / 2 ;
                for _i in 0..padding_count {
                    self.append_buffer.push(b' ');
                }

                self.append_buffer.append(message.into_bytes().as_mut());
            }

            if i < self.screenrows -1 {
                self.append_buffer.append(b"\r\n".to_vec().as_mut());
            }
        }
    }

    fn get_terminal_size(&mut self) -> Option<(u16,u16)> {
        self.stdout.write_all(b"\x1b[999C\x1b[999B").unwrap();
        self.get_cursor_position()
    }

    fn get_cursor_position(&mut self) -> Option<(u16,u16)> {
        self.stdout.write(b"\x1b[6n").unwrap();
        self.stdout.flush().unwrap();

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

        if buffer[0] != b'\x1b' || buffer[1] != b'[' { return None };
       
        let mut row: u16 = 0;
        let mut column: u16 = 0;

        if let Ok(s) = str::from_utf8(&buffer[1..7]) {
            row = s[1..=2].parse().unwrap();
            column = s[4..=5].parse().unwrap();
        }
        Some((row,column))
    }

    fn editor_move_cursor(&mut self, c: &EditorKey) {
        match c {
            EditorKey::ArrowUp    => self.cursor_y = self.cursor_y.saturating_sub(1), // w
            EditorKey::ArrowLeft  => self.cursor_x = self.cursor_x.saturating_sub(1), // a
            EditorKey::ArrowDown  => self.cursor_y = self.cursor_y.saturating_add(1), // s
            EditorKey::ArrowRight => self.cursor_x = self.cursor_x.saturating_add(1), // d
            EditorKey::Char(ch) => {
                match ch {
                    119 => self.cursor_y = self.cursor_y.saturating_sub(1),
                    97  => self.cursor_x = self.cursor_x.saturating_sub(1),
                    115 => self.cursor_y = self.cursor_y.saturating_add(1),
                    100 => self.cursor_x = self.cursor_x.saturating_add(1),
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.editor_refresh_screen();
        self.disable_raw_mode();
    }
}
