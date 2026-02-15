//! Demonstrates setting and then reading back the terminal window title.
//!
//! cargo run --example title-read

use std::io::{self, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, SetTitle},
};

const TEST_TITLE: &str = "Testing crossterm new function";

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Set the title
    execute!(stdout, SetTitle(TEST_TITLE))?;
    stdout.flush()?;

    // Give the terminal a moment to process the title change
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Read it back
    match terminal::title() {
        Ok(title) if title == TEST_TITLE => {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!("PASS: title matches: \"{title}\"\n")),
                ResetColor,
            )?;
        }
        Ok(title) => {
            execute!(
                stdout,
                SetForegroundColor(Color::Rgb { r: 255, g: 165, b: 0 }),
                Print(format!("MISMATCH: expected \"{TEST_TITLE}\", got \"{title}\"\n")),
                ResetColor,
            )?;
        }
        Err(e) => {
            eprintln!("FAIL: could not read title: {e}");
        }
    }

    Ok(())
}
