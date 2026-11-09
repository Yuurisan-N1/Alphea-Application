use std::io::{IsTerminal, Write};
use std::sync::{Mutex, Once, OnceLock};

static ARM: Once = Once::new();
static LOCK: OnceLock<Mutex<State>> = OnceLock::new();

pub const Z: &str = "\x1b[0m";

const ERASE: &str = "\x1b[2K";

#[derive(Default)]
struct State {
    shown: bool,
    len: usize,
    width: usize,
}

fn state() -> &'static Mutex<State> {
    LOCK.get_or_init(|| Mutex::new(State::default()))
}

pub fn arm() {
    ARM.call_once(enable_colour);
}

#[cfg(windows)]
fn enable_colour() {
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(which: u32) -> *mut core::ffi::c_void;
        fn GetConsoleMode(handle: *mut core::ffi::c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: *mut core::ffi::c_void, mode: *mut u32) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() {
            return;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return;
        }
        let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
}

#[cfg(not(windows))]
fn enable_colour() {}

fn erase_frames(st: &State) {
    let extra = if st.width > 1 {
        st.len.saturating_sub(1) / st.width
    } else {
        0
    };
    for _ in 0..extra {
        print!("\x1b[1A{}", ERASE);
    }
    print!("\r{}", ERASE);
}

pub fn emit(color: &str, msg: &str) {
    let mut st = state().lock().unwrap();
    if st.shown {
        erase_frames(&st);
        st.shown = false;
    }
    println!("{}{}{}", color, msg, Z);
    let _ = std::io::stdout().flush();
}

pub fn live(color: &str, msg: &str) {
    if !std::io::stdout().is_terminal() {
        println!("{}{}{}", color, msg, Z);
        let _ = std::io::stdout().flush();
        return;
    }
    let mut st = state().lock().unwrap();
    if st.shown {
        erase_frames(&st);
    }
    let width = term_width();
    let limit = if width > 1 {
        width - 1
    } else {
        usize::MAX
    };
    let text: String = msg.chars().take(limit).collect();
    print!("{}{}{}", color, text, Z);
    let _ = std::io::stdout().flush();
    st.shown = true;
    st.len = text.chars().count();
    st.width = width;
}

pub fn clear_live() {
    let mut st = state().lock().unwrap();
    if st.shown {
        erase_frames(&st);
        st.shown = false;
        let _ = std::io::stdout().flush();
    }
}

pub fn newline() {
    let mut st = state().lock().unwrap();
    st.shown = false;
    println!();
    let _ = std::io::stdout().flush();
}

pub fn blank() {
    let mut st = state().lock().unwrap();
    st.shown = false;
    println!();
    let _ = std::io::stdout().flush();
}

#[cfg(windows)]
fn term_width() -> usize {
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    #[repr(C)]
    #[derive(Default)]
    struct Coord {
        x: i16,
        y: i16,
    }
    #[repr(C)]
    #[derive(Default)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }
    #[repr(C)]
    #[derive(Default)]
    struct BufferInfo {
        size: Coord,
        cursor: Coord,
        attr: u16,
        win: SmallRect,
        maxwin: Coord,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(which: u32) -> *mut core::ffi::c_void;
        fn GetConsoleScreenBufferInfo(handle: *mut core::ffi::c_void, info: *mut BufferInfo) -> i32;
    }
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() {
            return 0;
        }
        let mut info = BufferInfo::default();
        if GetConsoleScreenBufferInfo(handle, &mut info) == 0 {
            return 0;
        }
        info.size.x.max(0) as usize
    }
}

#[cfg(not(windows))]
fn term_width() -> usize {
    unsafe {
        let mut size: libc::winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) != 0 {
            return 0;
        }
        size.ws_col.max(0) as usize
    }
}
