use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use std::io::{self, Read, Write};
use std::str;

pub fn get_size() -> io::Result<(usize, usize)> {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };
    Ok((ws.ws_col as usize, ws.ws_row as usize))
}

//pub fn get_cursor_position<R,W>(input: &mut R, output: &mut W) -> io::Result<(usize,usize)>
//where R: Read, W: Write {
//    output.write_all(b"\x1b[6n")?;
//    output.flush()?;
//
//    let mut buffer = Vec::<u8>::new();
//    input.read_to_end(&mut buffer)?;
//
//    if buffer[0] == b'\x1b' || buffer[1] == b'[' {  };
//
//    let mut x: usize = 0;
//    let mut y: usize = 0;
//
//    let bracket_position = &buffer.iter().position(|&x| x == 91).unwrap();
//    let semicolon_position = &buffer.iter().position(|&x| x == 59).unwrap();
//    let r_position = &buffer.iter().position(|&x| x == 82).unwrap();
//
//    if let Ok(s) = str::from_utf8(&buffer) {
//        y = s[bracket_position+1..*semicolon_position].parse().unwrap();
//        x = s[semicolon_position+1..*r_position].parse().unwrap();
//    }
//    Ok((x,y))
//}
