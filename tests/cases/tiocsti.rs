// smoelius: Based on: https://github.com/containers/bubblewrap/issues/142
// Thanks to @maxammann for the pointer.

macro_rules! i8_ptr {
    ($expr:expr) => {
        $expr as *const _ as *const i8
    };
}

fn main() {
    let cmd = "id\n";
    for c in cmd.chars() {
        if unsafe { libc::ioctl(libc::STDIN_FILENO, libc::TIOCSTI, i8_ptr!(&c)) } < 0 {
            unsafe { libc::perror(i8_ptr!(b"libc::ioctl\0")) };
            panic!();
        }
    }
}
