use libc::STDIN_FILENO;
use nix::unistd;
use termios::*;

fn main() {
    enable_raw_mode();
    let mut c: [u8;1] = [0];
    while unistd::read(STDIN_FILENO, &mut c).is_ok() 
        && c[0] != 113{
        
    }
    return;
}

fn enable_raw_mode() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();

    termios.c_lflag &= !(ECHO);

    tcsetattr(STDIN_FILENO, TCSANOW, &termios).unwrap();
}
