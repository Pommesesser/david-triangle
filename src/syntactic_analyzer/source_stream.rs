use std::io::Read;

pub const EOF: char = '\0';
pub const EOL: char = '\n';

pub struct SourceStream<R: Read> {
    reader: R,
    pub line: i32,
}

impl<R: Read> SourceStream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line: 1,
        }
    }

    pub fn get_source(&mut self) -> char {
        let mut buffer = [0u8; 1];

        match self.reader.read(&mut buffer) {
            Ok(1) => {
                let c = char::from(buffer[0]);

                if c == EOL {
                    self.line += 1;
                }

                c
            }

            _ => char::from(0)
        }
    }
}