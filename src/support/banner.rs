use chrono::Local;

pub fn print_banner() {
    crate::support::log::arm();
    clear_screen();
    set_title();

    let ascii = r#"__   __                _                     _
\ \ / /   _ _   _ _ __(_)___  __ _ _ __   __| | ___  ___ _   _
 \ V / | | | | | | '__| / __|/ _  | '_ \ / _  |/ _ \/ __| | | |
  | || |_| | |_| | |  | \__ \ (_| | | | | (_| |  __/\__ \ |_| |
  |_| \__,_|\__,_|_|  |_|___/\__,_|_| |_|\__,_|\___||___/\__,_|"#;

    println!("\x1b[1;36m{}\x1b[0m", ascii);
    println!();
    println!(
        "\x1b[1;35mWelcome to Yuuri, {}\x1b[0m",
        crate::support::consts::app::PROJECT
    );
    println!("\x1b[1;32mJoin our community: https://t.me/Y3YuYuYo\x1b[0m");
    println!(
        "\x1b[1;33mCurrent time: {}\x1b[0m\n",
        Local::now().format("%d-%m-%Y %H:%M:%S")
    );
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "cls"])
            .status();
    } else {
        print!("\x1b[2J\x1b[1;1H");
    }
}

fn set_title() {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn SetConsoleTitleW(name: *const u16) -> i32;
        }
        let wide: Vec<u16> = "Alphea Connect by : 佐賀県産 (YUURI)"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            SetConsoleTitleW(wide.as_ptr());
        }
    }
    #[cfg(not(windows))]
    {
        print!("\x1b]2;Alphea Connect by : 佐賀県産 (YUURI)\x07");
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
}
