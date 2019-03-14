//! This is a UNIX specific implementation for input related action.

use super::*;
use crate::sys::unix::{get_tty, read_char, read_char_raw};

use crossterm_utils::{csi, write, TerminalOutput};
use std::char;
use std::thread;

pub struct UnixInput;

impl UnixInput {
    pub fn new() -> UnixInput {
        UnixInput {}
    }
}

impl ITerminalInput for UnixInput {
    fn read_char(&self, stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<char> {
        let is_raw_screen = match stdout {
            Some(output) => output.is_in_raw_mode,
            None => false,
        };

        if is_raw_screen {
            read_char_raw()
        } else {
            read_char()
        }
    }

    fn read_async(
        &self,
        __stdout: &Option<&Arc<TerminalOutput>>,
    ) -> AsyncReader {
        AsyncReader::new(Box::new(move | event_tx, cancellation_token | {
            for i in get_tty().unwrap().bytes() {
                if event_tx.send(i.unwrap()).is_err() {
                    return;
                }

                if cancellation_token.load(Ordering::SeqCst) {
                    return;
                }
            }
        }))
    }

    fn read_until_async(
        &self,
        delimiter: u8,
        __stdout: &Option<&Arc<TerminalOutput>>,
    ) -> AsyncReader {
        AsyncReader::new(Box::new(move | event_tx, cancellation_token |  {
            for byte in get_tty().unwrap().bytes() {
                let byte = byte.unwrap();
                let end_of_stream = byte == delimiter;
                let send_error = event_tx.send(byte).is_err();

                if end_of_stream || send_error || cancellation_token.load(Ordering::SeqCst) {
                    return;
                }
            }
        }))
    }

    fn enable_mouse_mode(&self, stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<()> {
        write(
            stdout,
            format!(
                "{}h{}h{}h{}h",
                csi!("?1000"),
                csi!("?1002"),
                csi!("?1015"),
                csi!("?1006")
            ),
        )?;
        Ok(())
    }

    fn disable_mouse_mode(&self, stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<()> {
        write(
            stdout,
            format!(
                "{}l{}l{}l{}l",
                csi!("?1006"),
                csi!("?1015"),
                csi!("?1002"),
                csi!("?1000")
            ),
        )?;
        Ok(())
    }
}
