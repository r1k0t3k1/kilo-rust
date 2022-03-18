use std::io::Write;

pub struct EditorRow {
   pub chars: Vec<u8>,
   pub render: Vec<u8>,
}

const TAB_STOP: usize = 4;

impl EditorRow {
    pub fn insert_char(&mut self, char: u8, at: usize) {
       self.chars.insert(at, char); 
       self.render.insert(self.render_position(at), char);
       self.update();
    }

    pub fn delete_char(&mut self, at: usize) {
        if self.chars.len() > at {
           self.chars.remove(at); 
           self.update();
        }
    }

    pub fn append(&mut self, append_row: &mut Self) {
        self.chars.pop();
        self.chars.write(&append_row.chars).unwrap(); 
        self.update();
    }

    pub fn split(&mut self, at: usize) -> Self {
        EditorRow {
            chars: self.chars.split_off(at),
            render: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.render = Vec::new();
        for c in 0..self.chars.len() {
            if self.chars[c] == 9 {
                for _ in 0..TAB_STOP { self.render.push(32) }
            } else {
                self.render.push(self.chars[c]);
            }
        }
        self.render.push(0);
    }

    pub fn render_position(&self, cursor_x: usize) -> usize {
        let limit: usize;
        if self.chars.len() < cursor_x {
            limit = self.chars.len();
        } else {
            limit = cursor_x;
        }

        let tab_count = self.chars[0..limit]
            .iter()
            .filter(|&tab| *tab == 9).count();
        
        cursor_x + (TAB_STOP * tab_count) - tab_count
    }
}
