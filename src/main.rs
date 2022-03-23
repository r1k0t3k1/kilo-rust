use editor::Editor;
use key::ReadKey;
use std::io::{stdin, stdout, Write};
use std::{env, process};
use terminal_io::EnableRawMode;

mod csi;
mod editor;
mod key;
mod position;
mod row;
mod sys;
mod terminal_io;
mod window;

//const VERSION: &str = "0.0.1";

// エスケープシーケンス参考
// https://docs.microsoft.com/ja-jp/windows/console/console-virtual-terminal-sequences#input-sequences

fn main() {
    let mut t = stdout().enable_raw_mode().unwrap();
    let mut editor = Editor::new();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let filename = &args[1];
        editor.open_file(&filename).unwrap();
    } else {
        editor.open_empty();
    }

    editor.set_status_message("HELP: Ctrl-S => Save | Ctrl-Q => Quit".to_string());

    for c in stdin().keys() {
        let key = c.unwrap();
        if editor.process_keypress(&key) {
            t.output.write(csi::Csi::ClearScreen.to_string().as_bytes()).unwrap();
            t.output.write(csi::Csi::CursorToTopLeft.to_string().as_bytes()).unwrap();
            t.suspend_raw_mode().unwrap();
            process::exit(0);
        }
        editor.move_cursor(&key);
        editor.refresh_screen();
    }
}
