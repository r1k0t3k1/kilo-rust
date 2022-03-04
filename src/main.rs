use std::io::{stdin, stdout};
use std::{process, env};
use editor::Editor;
use terminal_io::EnableRawMode;
use key::ReadKey;

mod terminal_io;
mod editor;
mod window;
mod sys;
mod key;
mod row;

//const VERSION: &str = "0.0.1";

fn main() {
    let t = stdout().enable_raw_mode().unwrap(); 
    let mut editor = Editor::new();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 { 
        let filename = &args[1];
        editor.open_file(&filename).unwrap();
    }

    editor.set_status_message("HELP: Ctrl-Q => Quit".to_string());
    
    for c in stdin().keys() {
        let key = c.unwrap();
        match key {
            key::EditorKey::Ctrl(b'Q') => {
                t.suspend_raw_mode().unwrap();
                process::exit(0); 
            }
            key::EditorKey::PageUp => editor.process_keypress(&key),
            key::EditorKey::PageDown => editor.process_keypress(&key),
            key::EditorKey::End => editor.process_keypress(&key),
            _ => (),
        }
        editor.move_cursor(&key);
        editor.refresh_screen();
    }
}

