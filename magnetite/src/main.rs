use html::byte_stream_decoder::ByteStreamDecoder;
use html::input_stream_preprocessor::InputStreamPreprocessor;
use html::tokenizer::Token;
use html::tokenizer::Tokenizer;
use html::tree_constructor::*;
use magnetite::html;
use std::io::Cursor;

pub fn main() {
    let stream = Cursor::new(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <style>
            h1 {
                color: blue;
            }
        </style>
    </head>
    <body>
        <h1>
            Hello
        </h1>
        <p>
            Hello, Magnetite!
        </p>
    </body>
</html>"#,
    );
    let byte_stream_decoder = ByteStreamDecoder::new(stream);
    let input_stream_preprocessor = InputStreamPreprocessor::new(byte_stream_decoder).unwrap();
    let mut tree_constructor = TreeConstructor::new();
    let mut tokenizer = Tokenizer::new(input_stream_preprocessor, &mut tree_constructor);

    loop {
        if tokenizer.step().is_none() {
            break;
        }
    }

    println!("{:?}", tree_constructor.dom());
}
