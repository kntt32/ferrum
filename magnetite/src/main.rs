use css::tokenizer::Token as CssToken;
use css::tokenizer::Tokenizer as CssTokenizer;
use html::byte_stream_decoder::ByteStreamDecoder;
use html::input_stream_preprocessor::InputStreamPreprocessor;
use html::tokenizer::Tokenizer;
use html::tree_constructor::*;
use magnetite::css;
use magnetite::html;
use std::io::Cursor;

fn main() {
    css_demo();
}

#[allow(unused)]
fn css_demo() {
    let s = r#"
@import url(https://example.com/style.css);

@import url("https://example.com/quoted.css");
@import url('https://example.com/quoted-single.css');

@import url(   https://example.com/space.css   );

@import url("https://example.com/bad.css');

@import url();

@import url(https://example.com/comment.css); /* コメント */
        "#;

    let mut tokenizer = CssTokenizer::new(s);
    while let Some(t) = tokenizer.step() {
        println!("{:?}", t);
    }
    println!("{:?}", tokenizer);
}

#[allow(unused)]
fn html_demo() {
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
