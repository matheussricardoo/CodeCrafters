use std::io;
use std::os::unix::io::AsRawFd;

pub fn enable_raw_mode() {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();

    unsafe {
        libc::tcgetattr(stdin_fd, termios.as_mut_ptr());
        let mut termios = termios.assume_init();
        termios.c_lflag &= !(libc::ECHO | libc::ICANON);
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios);
    }
}

pub fn disable_raw_mode() {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();

    unsafe {
        libc::tcgetattr(stdin_fd, termios.as_mut_ptr());
        let mut termios = termios.assume_init();
        termios.c_lflag |= libc::ECHO | libc::ICANON;
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios);
    }
}
