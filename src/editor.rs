use std::io::{self, Stdin, Stdout, stdin, stdout, Write, BufReader, BufRead};
use std::fs::File;
use std::vec;

use crate::row::{EditorRow, self};
use crate::{key, window};

const TAB_STOP: usize = 4;

pub struct Editor {
    stdin: Stdin,
    stdout: Stdout,
    append_buffer: Vec<u8>,
    cursor_position: Position,
    render_cursor_position: Position, 
    rows: Vec<EditorRow>,
    offset: Position,
    window_size: Position,
    status_message: String,
    current_file_name: String,
}

pub struct Position {
    x: usize,
    y: usize,
}
impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
}

impl Editor {
   pub fn new() -> Editor {
        let mut stdin = stdin();
        let mut stdout = stdout();
        
        let window_size = window::get_size(&mut stdin, &mut stdout).unwrap();
        Editor {
            stdin: stdin,
            stdout: stdout,
            append_buffer: Vec::new(),
            cursor_position: Position::new(1, 0),
            render_cursor_position: Position::new(1, 0),
            rows: Vec::new(),
            offset: Position::new(0, 0),
            window_size: Position::new(window_size.0, window_size.1 - 2),
            status_message: "".to_string(),
            current_file_name: "[NO NAME]".to_string(),
        }
   } 

   pub fn open_file(&mut self, filename: &String) -> io::Result<()> {
       let file = File::open(&filename)?;
       self.current_file_name = filename.clone(); 
       for row in BufReader::new(file).lines() {
            let mut line = row?;
            line.push_str("\r");
            self.append_row(row::EditorRow {
                chars: line.into_bytes(),
                render: vec!(),
            });
       } 
       Ok(())
   }

   pub fn process_keypress(&mut self, key: &key::EditorKey) {
        match key {
            &key::EditorKey::PageUp => self.cursor_position.y = self.offset.y,
            &key::EditorKey::PageDown => {
                self.cursor_position.y = self.offset.y + self.window_size.y - 1;
                if self.cursor_position.y > self.rows.len() { self.cursor_position.y = self.rows.len() };
            },
            &key::EditorKey::End => {
                if self.cursor_position.y == self.rows.len() {
                    self.cursor_position.x = self.rows[self.cursor_position.y].chars.len()
                };
            },
            _ => (),
        }
   }

   pub fn move_cursor(&mut self, key: &key::EditorKey) {
        let limit_x;
        let limit_y;
        if self.rows.len() == 0 { 
            limit_x = 0;
            limit_y = 0;
        } else {
            limit_x = if self.cursor_position.y == self.rows.len() {
                1
            } else {
                self.rows[self.cursor_position.y].chars.len() - 1
            };

            limit_y =self.rows.len() - 1;
        }

        match key {
            key::EditorKey::ArrowLeft  => {
                if self.cursor_position.x == 1 {
                    if self.cursor_position.y > 0 {
                        self.cursor_position.y -= 1;
                        self.cursor_position.x = self.rows[self.cursor_position.y].chars.len();
                    }
                }else {
                    self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
                }
            }
            key::EditorKey::ArrowRight => {
                if self.cursor_position.y < self.rows.len() - 1 {
                    if self.cursor_position.x < limit_x { 
                        self.cursor_position.x += 1 
                    } else {
                        self.cursor_position.y += 1;
                        self.cursor_position.x = 1;
                    }
                }
            }
            key::EditorKey::ArrowUp    => {
                if self.rows.len() == self.cursor_position.y { 
                    self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
                    return;
                }
                if self.cursor_position.x > self.rows[self.cursor_position.y.saturating_sub(1)].chars.len() {
                    self.cursor_position.x = self.rows[self.cursor_position.y.saturating_sub(1)].chars.len();
                    self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
                    return;
                }
                self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
            }
            key::EditorKey::ArrowDown  => {
                if self.cursor_position.y < limit_y { 
                    self.cursor_position.y += 1;
                    if self.cursor_position.x > self.rows[self.cursor_position.y].chars.len() {
                        self.cursor_position.x = self.rows[self.cursor_position.y].chars.len();
                    }
                };
            },
            _ => (),
        }
   }

   pub fn refresh_screen(&mut self) {
       self.scroll(); 
       self.append_buffer.append(b"\x1b[?25l\x1b[H".to_vec().as_mut());
       self.draw_rows();
       self.draw_status_bar();
       self.append_buffer.append(format!("\x1b[{};{}H",
               self.cursor_position.y - self.offset.y + 1,
               self.render_cursor_position.x - self.offset.x + 1)
           .as_bytes()
           .to_vec()
           .as_mut());
        self.append_buffer.append(b"\x1b[?25h".to_vec().as_mut());
        self.stdout.write_all(self.append_buffer.as_slice()).unwrap();
        self.stdout.flush().unwrap();
        self.append_buffer = vec!();
   }

   pub fn draw_rows(&mut self) {
        for i in 0..self.window_size.y {
            self.append_buffer.append(b"\x1b[48;5;236m~\x1b[m\x1b[K".to_vec().as_mut());
            let file_row = i + self.offset.y;
            if file_row >= self.rows.len() {
                if i >= self.rows.len() {
                    if self.rows.len() == 0 && i == self.window_size.y / 3 {
                        let message = format!("riko editor -- version 0.0.1");
                        let padding = (self.window_size.x - message.len()) / 2;
                        for _ in 0..padding {
                            self.append_buffer.push(b' ');
                        }
                        self.append_buffer.append(message.into_bytes().as_mut());
                    } 
                } else {
                    self.append_buffer.append(&mut self.rows[i].chars.clone());
                }
            } else {
                let mut len = self.rows[file_row].render.len().saturating_sub(self.offset.x);
                if len > self.window_size.x { len = self.window_size.x }

                let end = (self.offset.x + len).saturating_sub(1);
                self.append_buffer.append(self.rows[file_row].render[self.offset.x..end].to_vec().as_mut());
            }
            self.append_buffer.append(b"\r\n".to_vec().as_mut());
        }
   }

    fn draw_status_bar(&mut self) {
        self.append_buffer.append(b"\x1b[48;5;245m".to_vec().as_mut());
        
        let mut status_text = format!("{}:r{}:c{}",
            self.current_file_name,
            self.cursor_position.y,
            self.render_cursor_position.x,
        );
        for _ in 0..self.window_size.x - status_text.len(){
            status_text.push(' ');
        }
        self.append_buffer.append(status_text.as_bytes().to_vec().as_mut());
        self.append_buffer.append(b"\r\n".to_vec().as_mut());
        self.append_buffer.append(b"\x1b[m".to_vec().as_mut());

        self.append_buffer.append(b"\x1b[K".to_vec().as_mut());
        self.append_buffer.append(self.status_message.as_bytes().to_vec().as_mut());
    }

    pub fn set_status_message(&mut self, message: String) {
       self.status_message = message; 
    }

   fn scroll(&mut self) {
        self.render_cursor_position.x = 0;
        if self.cursor_position.y < self.rows.len() {
            self.cursol2render_cursol();
        }
        if self.cursor_position.y < self.offset.y {
            self.offset.y = self.cursor_position.y;
        }
        if self.cursor_position.y >= self.offset.y + self.window_size.y {
            self.offset.y = self.cursor_position.y - self.window_size.y + 1;
        }
        if self.render_cursor_position.x < self.offset.x {
            self.offset.x = self.render_cursor_position.x;
        }
        if self.render_cursor_position.x >= self.offset.x + self.window_size.x {
            self.offset.x = self.render_cursor_position.x - self.window_size.x + 1;
        }

   }

   fn append_row(&mut self, row: EditorRow) {
       self.rows.push(row);
       self.update_row();
   }

   fn update_row(&mut self) {
       let last = self.rows.last_mut().unwrap();

       for c in 0..last.chars.len() {
            if last.chars[c] == 9 {
                for _ in 0..TAB_STOP { last.render.push(32) }
            } else {
                last.render.push(last.chars[c]);
            }
       }
       last.render.push(0);
   }

   fn cursol2render_cursol(&mut self) {
        let limit: usize;
        if self.rows[self.cursor_position.y].chars.len() < self.cursor_position.x { 
            limit = self.rows[self.cursor_position.y].chars.len();
        } else {
            limit = self.cursor_position.x;
        }
        let tab_count = self.rows[self.cursor_position.y].chars[0..limit]
            .iter()
            .filter(|&tab| *tab == 9).count();
        let rx = self.cursor_position.x + (TAB_STOP * tab_count) - tab_count;
        self.render_cursor_position.x = rx;
   }
}
