use std::io::Write;

use open_dmx::DMXSerial;

use crate::{lexer::Lexer, parser::Parser};

pub mod lexer;
pub mod parser;

// const SERIAL_PORT: &str = "/dev/tty.usbserial-A10KPDBZ";
const SERIAL_PORT: &str = "/dev/ttys001";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dmx = DMXSerial::open_sync(SERIAL_PORT).unwrap();

    loop {
        print!("[demex] > ");
        std::io::stdout().flush().unwrap();

        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd)?;

        let mut l = Lexer::new(&cmd);
        let tokens = l.tokenize()?;

        let mut p = Parser::new(&tokens);
        let action = p.parse()?;

        let res = action.run(&mut dmx)?;

        if res.should_update {
            dmx.update().unwrap();
        }
    }
}
