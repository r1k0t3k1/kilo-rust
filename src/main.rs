use libc::STDIN_FILENO;
use nix::unistd;

fn main() {
    let mut c: [u8;1] = [0];
    while unistd::read(STDIN_FILENO, &mut c).is_ok() {
        
    }
    return;
}

