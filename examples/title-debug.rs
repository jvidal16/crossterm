//! Debug tool: sends CSI 21 t and prints raw response bytes.
//! Bypasses crossterm's event system to show exactly what the terminal sends.
//!
//! cargo run --example title-debug

use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::time::Duration;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() -> io::Result<()> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    enable_raw_mode()?;

    // Set a known title
    tty.write_all(b"\x1B]0;Debug Title Test\x07")?;
    tty.flush()?;
    std::thread::sleep(Duration::from_millis(100));

    // Drain any pending input
    set_nonblocking(&tty, true)?;
    {
        let mut drain = [0u8; 256];
        while tty.read(&mut drain).unwrap_or(0) > 0 {}
    }

    // Send the title query
    tty.write_all(b"\x1B[21t")?;
    tty.flush()?;

    // Read whatever comes back (non-blocking poll loop, 3s timeout)
    let mut all_bytes = Vec::new();
    let mut buf = [0u8; 256];
    let deadline = std::time::Instant::now() + Duration::from_secs(3);

    while std::time::Instant::now() < deadline {
        match tty.read(&mut buf) {
            Ok(n) if n > 0 => {
                all_bytes.extend_from_slice(&buf[..n]);
                if all_bytes.windows(2).any(|w| w == b"\x1B\\")
                    || all_bytes.iter().any(|&b| b == 0x07)
                {
                    break;
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    set_nonblocking(&tty, false)?;
    disable_raw_mode()?;

    if all_bytes.is_empty() {
        println!("No bytes received. Terminal does not respond to CSI 21 t.");
    } else {
        print!("Received {} bytes:\r\n", all_bytes.len());
        print!("  Hex:");
        for b in &all_bytes {
            print!(" {:02X}", b);
        }
        print!("\r\n");
        print!("  Str: ");
        for b in &all_bytes {
            if b.is_ascii_graphic() || *b == b' ' {
                print!("{}", *b as char);
            } else {
                print!("\\x{:02X}", b);
            }
        }
        print!("\r\n");
    }

    Ok(())
}

fn set_nonblocking(fd: &impl AsFd, nonblocking: bool) -> io::Result<()> {
    let mut flags = rustix::fs::fcntl_getfl(fd)?;
    if nonblocking {
        flags |= rustix::fs::OFlags::NONBLOCK;
    } else {
        flags &= !rustix::fs::OFlags::NONBLOCK;
    }
    rustix::fs::fcntl_setfl(fd, flags)?;
    Ok(())
}
