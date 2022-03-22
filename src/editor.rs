use std::io::{self, Stdout, stdout, Write, BufReader, BufRead};
use std::fs::File;
use std::{vec, usize};

use crate::row::{EditorRow, self};
use crate::{key, window, position::Position};

pub struct Editor {
    stdout: Stdout,
    append_buffer: Vec<u8>,
    cursor_position: Position,
    render_cursor_position: Position, 
    rows: Vec<EditorRow>,
    offset: Position,
    window_size: Position,
    status_message: String,
    current_file_name: String,
    is_dirty: bool,
    e_key_history: Vec<key::EditorKey>,
}

impl Editor {
   pub fn new() -> Editor {
        let stdout = stdout();
        
        let window_size = window::get_size().unwrap();
        Editor {
            stdout,
            append_buffer: Vec::new(),
            cursor_position: Position::new(0, 0),
            render_cursor_position: Position::new(0, 0),
            rows: Vec::new(),
            offset: Position::new(0, 0),
            window_size: Position::new(window_size.0, window_size.1 - 2),
            status_message: "".to_string(),
            current_file_name: "[NO NAME]".to_string(),
            is_dirty: false,
            e_key_history: Vec::new(),
        }
   } 

   pub fn open_file(&mut self, filename: &String) -> io::Result<()> {
       let file = File::open(&filename)?;
       self.current_file_name = filename.clone(); 
       for row in BufReader::new(file).lines() {
            let line_with_lf = row?.into_bytes();
            //line_with_lf.push(b'\n');
            self.insert_row(self.rows.len(), row::EditorRow {
                chars: line_with_lf,
                render: vec!(),
            });
       } 
       self.is_dirty = false;
       Ok(())
   }

   fn save(&mut self) -> io::Result<()>{
        if String::is_empty(&self.current_file_name) { return Ok(()); }
        let mut file = File::create(&self.current_file_name)?;
        
        for r in &mut self.rows {
            r.chars.push(b'\n');
            //if let Some(index) = &r.chars.iter().position(|c| *c == b'\n') {
            //    let _ = std::mem::replace(&mut r.chars[*index], b'\n');
            //}
            file.write_all(r.chars.as_slice())?;
        }
        self.is_dirty = false;
        Ok(())
   }

   pub fn process_keypress(&mut self, key: &key::EditorKey) -> bool {
       let mut allow_exit = false;
        match key {
            key::EditorKey::Enter => self.insert_newline(),
            key::EditorKey::Char(c) => self.insert_char(*c),
            key::EditorKey::BackSpace => self.backspace(),
            key::EditorKey::PageUp => self.cursor_position.y = self.offset.y,
            key::EditorKey::PageDown => {
                self.cursor_position.y = self.offset.y + self.window_size.y - 1;
                if self.cursor_position.y > self.rows.len() { self.cursor_position.y = self.rows.len() };
            },
            key::EditorKey::End => { if self.cursor_position.y == self.rows.len() {
                    self.cursor_position.x = self.rows[self.cursor_position.y].chars.len()
                };
            },
            key::EditorKey::Ctrl(b'Q') => {
                if self.is_dirty {
                    match self.e_key_history.last() {
                        Some(key::EditorKey::Ctrl(b'Q')) => allow_exit = true,
                        _ => {
                            self.set_status_message("WARNING!! File has unsaved changes. Press Ctrl-Q again to quit.".to_string());
                            allow_exit = false;
                        }
                    }
                } else {
                    allow_exit = true;
                }
            },
            key::EditorKey::Ctrl(b'S') => {
                match self.save() {
                    Ok(()) => self.set_status_message("Written to disk".to_string()),        
                    Err(_) => self.set_status_message("Can not save! I/O Error".to_string()),        
                }
            }
            key::EditorKey::Ctrl(b'L') => (),
            key::EditorKey::Ctrl(b'H') => (),
            key::EditorKey::Null => return false,
            _ => (),
        }
        self.e_key_history.push(key.clone());
        allow_exit
   }

   pub fn move_cursor(&mut self, key: &key::EditorKey) {
        let limit_x;
        let limit_y;
        if self.rows.len() == 0 { 
            limit_x = 0;
            limit_y = 0;
        } else {
            limit_x = if self.cursor_position.y == self.rows.len() {
                0
            } else {
                self.rows[self.cursor_position.y].chars.len()
            };

            limit_y =self.rows.len() - 1;
        }

        match key {
            &key::EditorKey::ArrowLeft  => {
                if self.cursor_position.x == 0 {
                    if self.cursor_position.y > 0 {
                        self.cursor_position.y -= 1;
                        self.cursor_position.x = self.rows[self.cursor_position.y].chars.len();
                    }
                }else {
                    self.cursor_position -= Position::new(1,0);
                }
            }
            &key::EditorKey::ArrowRight => {
                if self.cursor_position.y >= limit_y && self.cursor_position.x >= limit_x {
                    return;
                }
                if self.cursor_position.x < limit_x { 
                    self.cursor_position.x += 1 
                } else {
                    self.cursor_position.y += 1;
                    self.cursor_position.x = 0;
                }
            }
            &key::EditorKey::ArrowUp => {
                if self.rows.len() == self.cursor_position.y { 
                    self.cursor_position -= Position::new(0,1);
                    return;
                }
                if self.cursor_position.x > self.rows[self.cursor_position.y.saturating_sub(1)].chars.len() {
                    self.cursor_position.x = self.rows[self.cursor_position.y.saturating_sub(1)].chars.len();
                    self.cursor_position -= Position::new(0,1);
                    return;
                }
                self.cursor_position -= Position::new(0,1);
            }
            &key::EditorKey::ArrowDown  => {
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
            self.append_buffer.append(b"\x1b[K".to_vec().as_mut());
            let file_row = i + self.offset.y;
            if file_row >= self.rows.len() {
                self.append_buffer.append(b"\x1b[48;5;236m~\x1b[m".to_vec().as_mut());
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

        let mut status_text = format!("{}{}: (cx{}, cy{}): (rcx{}, rcy{}): lc:{}",
            self.current_file_name,
            if let true = self.is_dirty { "(modified)" } else { "" },
            self.cursor_position.x,
            self.cursor_position.y,
            self.render_cursor_position.x,
            self.render_cursor_position.y,
            self.rows.len(),
        );
        for _ in 0..self.window_size.x - status_text.len(){
            status_text.push(' ');
        }
        self.append_buffer.append(status_text.as_bytes().to_vec().as_mut());
        self.append_buffer.append(b"\x1b[m".to_vec().as_mut());

        self.append_buffer.append(b"\r\n".to_vec().as_mut());
        self.append_buffer.append(b"\x1b[K".to_vec().as_mut());
        self.append_buffer.append(self.status_message.as_bytes().to_vec().as_mut());
    }

    pub fn set_status_message(&mut self, message: String) {
       self.status_message = message; 
    }

   fn scroll(&mut self) {
        self.render_cursor_position.x = 0;
        if self.cursor_position.y < self.rows.len() {
            self.render_cursor_position.x = self.rows[self.cursor_position.y]
                .render_position(self.cursor_position.x);
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

   fn insert_row(&mut self, at: usize, row: EditorRow) {
       self.rows.insert(at, row);
       self.rows[at].update();
       self.is_dirty = true;
   }

   fn insert_newline(&mut self) {
        if self.rows[self.cursor_position.y].chars.len() == 0 {
            self.insert_row(self.cursor_position.y + 1, EditorRow { chars: vec!(), render: vec!() });
        } else if self.cursor_position.x == self.rows[self.cursor_position.y].chars.len() {
            self.insert_row(self.cursor_position.y + 1, EditorRow { chars: vec!(), render: vec!() });
        } else {
            let r = self.rows[self.cursor_position.y]
                .split(self.cursor_position.x);
            if r.chars.len() != 0 {
                self.rows[self.cursor_position.y].update();
                self.insert_row(self.cursor_position.y + 1, r);
            }
        }
        self.cursor_position.x = 0;
        self.cursor_position.y += 1;
    }

   fn insert_char(&mut self, char: u8) {
        self.rows[self.cursor_position.y]
            .insert_char(char, self.cursor_position.x);
        self.cursor_position.x += 1;
        self.render_cursor_position.x += 1;
        self.is_dirty = true;
   }

   fn backspace(&mut self) {
       if self.cursor_position.x >= 1 {
            self.rows[self.cursor_position.y]
                .delete_char(self.cursor_position.x.saturating_sub(1));
            self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
            self.render_cursor_position.x = self.rows[self.cursor_position.y]
                .render_position(self.cursor_position.x);
       } else {
            if self.cursor_position.y == 0 { return }
            let mut mv = self.rows.remove(self.cursor_position.y);
            self.cursor_position.y -= 1;
            self.cursor_position.x = self.rows[self.cursor_position.y].chars.len();
            self.rows[self.cursor_position.y].append(&mut mv);
            self.rows[self.cursor_position.y].update();
       }
        self.is_dirty = true;
   }
}
