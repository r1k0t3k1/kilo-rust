use std::fs::File;
use std::io::{stdin, stdout, Read, Write, Stdin, Stdout, Error, BufReader, BufRead };
use std::{process, str, u8, env, usize};
use terminal_io::EnableRawMode;
use key::

mod terminal_io;
mod window;
mod sys;
mod key;
mod row;

const VERSION: &str = "0.0.1";

fn main() {
    let t = stdout().enable_raw_mode().unwrap(); 
    let mut raw_terminal = RawTerminal {
            stdin: stdin(),
            stdout: t,
            append_buffer: vec![],
            screencols: 0,
            screenrows: 0,
            cursor_x: 0,
            cursor_y: 0,
            row: Vec::<row::EditorRow>::new(),
            row_offset: 0,
            column_offset: 0,
        };

    let screensize = window::get_size(&mut raw_terminal.stdin, &mut raw_terminal.stdout.output).unwrap();
    raw_terminal.screenrows = screensize.0;
    raw_terminal.screencols = screensize.1;

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 { 
        let filename = &args[1];
        raw_terminal.editor_open(&filename);
    }
        

    loop {
        raw_terminal.editor_refresh_screen();
        raw_terminal.editor_process_keypress();
    }
}

pub struct RawTerminal {
    stdin: Stdin,
    stdout: terminal_io::RawTerminal<Stdout>,
    append_buffer: Vec<u8>,
    screencols: usize,
    screenrows: usize,
    cursor_x: usize,
    cursor_y: usize,
    row: Vec<row::EditorRow>,
    row_offset: usize,
    column_offset: usize,
}

impl RawTerminal {
    fn editor_read_key(&mut self) -> Result<key::EditorKey,Error> {
       let mut c = [0_u8;4]; 
       self.stdin.read(&mut c)?;
       if c[0] == 27 {
            if c[1] == 91 {
                if c[3] == 126 {
                    match c[2] {
                        49 => return Ok(key::EditorKey::Home),
                        51 => return Ok(key::EditorKey::Delete),
                        52 => return Ok(key::EditorKey::End),
                        53 => return Ok(key::EditorKey::PageUp),
                        54 => return Ok(key::EditorKey::PageDown),
                        55 => return Ok(key::EditorKey::Home),
                        56 => return Ok(key::EditorKey::End),
                        _ => (),
                    }
                }
                match c[2] {
                    65 => return Ok(key::EditorKey::ArrowUp),
                    66 => return Ok(key::EditorKey::ArrowDown),
                    67 => return Ok(key::EditorKey::ArrowRight),
                    68 => return Ok(key::EditorKey::ArrowLeft),
                    70 => return Ok(key::EditorKey::End),
                    72 => return Ok(key::EditorKey::Home),
                    _  => (),
                }
            }
            return Ok(key::EditorKey::Escape);
       } else if c[0] == 79 {
           match c[1] {
                70 => return Ok(key::EditorKey::End),
                72 => return Ok(key::EditorKey::Home),
                _  => (),
           }
       } 
       Ok(key::EditorKey::Char(c[0]))
    }

    fn editor_process_keypress(&mut self) {
        let c = self.editor_read_key().unwrap();
        if c == key::ctrl('q') {
            self.stdout.suspend_raw_mode().unwrap();
            process::exit(0);
        }
        self.editor_move_cursor(&c);
        match c {
            key::EditorKey::Char(ch) => {
                let byte = [ch;1];
                self.stdout.output.write(&byte).unwrap();}
            key::EditorKey::Home => self.cursor_x = 0,
            key::EditorKey::End  => self.cursor_x = self.screencols - 1,
            _ => (),
        }
    }

    fn editor_refresh_screen(&mut self) {
        self.editor_scroll();
        self.append_buffer.append(b"\x1b[?25l\x1b[H".to_vec().as_mut());
        self.editor_draw_rows();
        self.append_buffer.append(format!("\x1b[{};{}H",
                self.cursor_y - self.row_offset + 1,
                self.cursor_x - self.column_offset + 1)
            .as_bytes()
            .to_vec()
            .as_mut());
        self.append_buffer.append(b"\x1b[?25h".to_vec().as_mut());
        self.stdout.output.write_all(self.append_buffer.as_slice()).unwrap();
        self.stdout.output.flush().unwrap();
    }

    fn editor_draw_rows(&mut self) {
        for i in 0..self.screenrows {
            self.append_buffer.append(b"~\x1b[K".to_vec().as_mut());
            let file_row = i + self.row_offset;
            if i + self.row_offset >= self.row.len() {
                if i >= self.row.len() {
                    if self.row.len() == 0 && i == self.screenrows / 3 {
                        let message = format!("riko editor -- version {}", VERSION); 

                        let padding_count = (self.screencols - message.len()) / 2;
                        for _i in 0..padding_count {
                            self.append_buffer.push(b' ');
                        }

                        self.append_buffer.append(message.into_bytes().as_mut());
                    }
                } else {
                    self.append_buffer.append(&mut self.row[i].chars.clone());
                }
            } else {
                let mut len = self.row[file_row].chars.len().saturating_sub(self.column_offset);
                if len > self.screencols { len = self.screencols }

                let end = self.column_offset + len - 1;

                if self.column_offset < self.row[file_row].chars.len() {
                    let offset_text = &mut self.row[file_row]
                        .chars
                        .clone()[(self.column_offset)..end]
                        .to_vec();
                    self.append_buffer.append(offset_text);
                } 
            }

            if i < self.screenrows -1 {
                self.append_buffer.append(b"\r\n".to_vec().as_mut());
            }
        }
    }

    fn editor_move_cursor(&mut self, c: &key::EditorKey) {
        let limit_x = if self.cursor_y == self.row.len() {
            0
        } else {
            self.row[self.cursor_y].chars.len() - 1
        };

        let limit_y =self.row.len();

        match c {
            key::EditorKey::ArrowLeft  => self.cursor_x = self.cursor_x.saturating_sub(1), 
            key::EditorKey::ArrowRight => if self.cursor_x < limit_x { self.cursor_x += 1 },
            key::EditorKey::ArrowUp    => self.cursor_y = self.cursor_y.saturating_sub(1),
            key::EditorKey::ArrowDown  => if self.cursor_y < limit_y { self.cursor_y += 1 },
            key::EditorKey::PageDown => {
                let mut times = self.screenrows;
                while times > 0 {
                    self.cursor_y = self.cursor_y.saturating_sub(1);
                    times -= 1;
                };
            }
            _ => (),
        }
    }

    fn editor_open(&mut self, filename: &String) {
        let file = File::open(filename)
            .expect("file open error");

        for result in BufReader::new(file).lines() {
           let mut l = result.unwrap();  
           l.push_str("\r");
           self.editor_append_row(l);
        }
    }

    fn editor_append_row(&mut self, row_str: String) {
        let mut r = row::EditorRow { chars: Vec::new() };
        r.chars = row_str.into_bytes();
        r.chars.push(0);
        self.row.push(r);
    }

    fn editor_scroll(&mut self) {
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }
        if self.cursor_y >= self.row_offset + self.screenrows {
            self.row_offset = self.cursor_y - self.screenrows + 1; 
        }
        if self.cursor_x < self.column_offset { 
            self.column_offset = self.cursor_x;
        }
        if self.cursor_x >= self.column_offset + self.screencols { 
            self.column_offset = self.cursor_x - self.screencols + 1;
        }
    }
}
