use super::byte_stream_decoder::ByteStreamDecoder;
use std::io::Error as IoError;
use std::io::Read;

pub struct InputStreamPreprocessor {
    string: String,
}

impl InputStreamPreprocessor {
    pub fn new<S: Read>(mut decoder: ByteStreamDecoder<S>) -> Result<Self, IoError> {
        Ok(Self {
            string: decoder.decode()?,
        })
    }

    pub fn preprocess(mut self) -> String {
        self.normalize_crlf();
        self.delete_bom();
        self.string
    }

    fn normalize_crlf(&mut self) {
        self.string = self.string.replace("\r\n", "\n").replace("\r", "\n");
    }

    fn delete_bom(&mut self) {
        const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];
        if &self.string.as_bytes()[..3] == UTF8_BOM {
            self.string.replace_range(..3, "");
        }
    }
}
