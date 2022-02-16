use std::io::{stdin, stdout, Write, Read};
use std::io::{Stdin,Stdout};
use std::str;

use crate::EditorRow;
use crate::RawTerminal;

pub struct Editor {
    stdin: Stdin,
    stdout: Stdout,
    append_buffer: Vec<u8>,
    cursor_position: Position,
    rows: Vec<EditorRow>,
    offset: Position,
}

pub struct Position {
    x: usize,
    y: usize,
}

impl Editor {
   fn new() -> Editor {
        Editor {
            stdin: stdin(),
            stdout: stdout(),
            append_buffer: Vec::new(),
            cursor_position: Position{ x:0, y:0 },
            rows: Vec::new(),
            offset: Position{ x:0, y:0 },
        }
   } 

   fn get_terminal_size(&mut self) -> Option<Position> {
       self.stdout.write_all(b"").unwrap();
       self.stdout.flush().unwrap();
       self.get_cursor_position()
   }

   fn get_cursor_position(&mut self) -> Option<Position> {
        self.stdout.write(b"x1b[6n").unwrap();
        self.stdout.flush().unwrap();
        
        let mut buffer = Vec::<u8>::new();
        self.stdin.read_to_end(&mut buffer);

        if buffer[0] != b'\x1b' || buffer[1] != b'[' { return None };

        let bracket_position = &buffer.iter().position(|&x| x == 91).unwrap();
        let semicolon_position = &buffer.iter().position(|&x| x == 59).unwrap();
        let r_position = &buffer.iter().position(|&x| x == 82).unwrap();

        let mut x = 0;
        let mut y = 0;
        if let Ok(s) = str::from_utf8(&buffer) {
            x = s[bracket_position + 1..*semicolon_position].parse().unwrap();
            y = s[bracket_position + 1..*semicolon_position].parse().unwrap();
        }

        Some(Position { x, y })
   }
}

