use magnetite::byte_stream_decoder::ByteStreamDecoder;
use magnetite::input_stream_preprocessor::InputStreamPreprocessor;
use magnetite::tokenizer::Token;
use magnetite::tokenizer::Tokenizer;
use std::io::Cursor;

pub fn main() {
    let stream = Cursor::new("<h1>Hello, World!</h1><p>This is magnetie!</p>");
    let byte_stream_decoder = ByteStreamDecoder::new(stream);
    let input_stream_preprocessor = InputStreamPreprocessor::new(byte_stream_decoder).unwrap();
    let mut tokenizer = Tokenizer::new(input_stream_preprocessor);

    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    loop {
        tokenizer.step(
            &mut |t| {
                println!("{:?}", t);
                tokens.push(t)
            },
            &mut |e| errors.push(e),
        );
        if tokens.last() == Some(&Token::Eof) {
            break;
        }
    }

    println!("tokens: {:?}", tokens);
    println!("errors: {:?}", errors);
}
