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
    let mut tokenizer = Tokenizer::new(input_stream_preprocessor);
    let mut tree_constructor = TreeConstructor::new();

    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    'a: loop {
        tokenizer.step(&mut |t| tokens.push(t), &mut |e| errors.push(e), &|| {
            tree_constructor.adjusted_current_node_namespace()
        });
        for token in &tokens {
            println!("{:?}", token);
            tree_constructor.handle_token(token.clone());
            if tree_constructor.errors().len() != 0 {
                println!("{:?}", tree_constructor.errors());
                break 'a;
            }
            if token == &Token::Eof {
                break 'a;
            }
        }
        tokens.clear();
    }

    println!("{:?}", tree_constructor.dom());
}
