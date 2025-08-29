use std::io::Error as IoError;
use std::io::Read;

pub struct ByteStreamDecoder<S: Read> {
    stream: S,
}

impl<S: Read> ByteStreamDecoder<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub fn decode(&mut self) -> Result<String, IoError> {
        let mut vec = Vec::new();
        if let Err(e) = self.stream.read_to_end(&mut vec) {
            return Err(e);
        }

        let string_cow = String::from_utf8_lossy(&vec);
        Ok(string_cow.into_owned())
    }
}
