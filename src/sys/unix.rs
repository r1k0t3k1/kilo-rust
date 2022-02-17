use termios::*;
use std::io::{self, stdin};
use std::os::unix::io::AsRawFd;

pub fn get_terminal_setting() -> io::Result<Termios> {
    let fd = stdin().as_raw_fd();
    Termios::from_fd(fd)
}

pub fn set_terminal_setting(termios: &Termios) -> io::Result<()> {
    let fd = stdin().as_raw_fd();
    tcsetattr(fd, TCSAFLUSH, &termios)?;
    Ok(())
}

pub fn get_raw_terminal_setting() -> io::Result<Termios> {
    let fd = stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;

    // echo off
    termios.c_lflag &= !(ECHO);
    // turn off canonical mode
    termios.c_lflag &= !(ICANON);
    // turn off Ctrl-C and Ctrl-Z signal
    termios.c_lflag &= !(ISIG);
    // disable Ctrl-S and Ctrl-Q
    termios.c_iflag &= !(IXON);
    // disable Ctrl-V
    termios.c_lflag &= !(IEXTEN);
    // fix Ctrl-M
    termios.c_iflag &= !(ICRNL);
    // turn off all output processing
    termios.c_oflag &= !(OPOST);
    // miscellaneous flags
    termios.c_iflag &= !(BRKINT);
    termios.c_iflag &= !(INPCK);
    termios.c_iflag &= !(ISTRIP);
    termios.c_cflag |= CS8;
    // timeout for read
    termios.c_cc[VMIN] = 0;
    termios.c_cc[VTIME] = 1;
 
    tcsetattr(fd, TCSAFLUSH, &termios)?;
    
    Ok(termios)
}
