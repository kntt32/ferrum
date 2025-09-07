use css::tokenizer::Token as CssToken;
use css::tokenizer::Tokenizer as CssTokenizer;
use html::byte_stream_decoder::ByteStreamDecoder;
use html::input_stream_preprocessor::InputStreamPreprocessor;
use html::tokenizer::Tokenizer;
use html::tree_constructor::*;
use magnetite::arena::*;
use magnetite::css;
use magnetite::html;
use magnetite::render::RenderArena;
use std::io::Cursor;

fn main() {
    html_demo();
}

#[allow(unused)]
fn arena_demo() {
    let mut arena: Arena<i32> = Arena::new();
    let root = arena.push(1);
    let child1 = arena.insert_child(root, 2);
    let child2 = arena.insert_child(root, 3);
    arena.insert_child(root, 4);
    arena.insert_child(child2, 5);
    println!("{:?}", arena);
    arena.unlink(child2);
    println!("{:?}", arena);
    arena.unlink(child1);
    println!("{:?}", arena);
}

#[allow(unused)]
fn css_demo() {
    let s = r#"/* コメント */
@import url("https://example.com/style.css");

:root {
    --main-color: #ff00ff;
    --font-size: 16px;
    width: calc(100% - 20px);
    height: 50vh;
}
        "#;

    let mut tokenizer = CssTokenizer::new(s);
    while let Some(t) = tokenizer.step() {
        println!("{:?}", t);
    }
    println!("{:?}", tokenizer);
}

#[allow(unused)]
fn html_demo() {
    /*
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
    );*/
    let stream = Cursor::new(
        r#"
<!DOCTYPE html>
<html>
    <body>
        <h1>
            Hello
        </h1>
    </body>
</html>
        "#,
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

    let dom = tree_constructor.take_dom();
    println!("{:?}", dom);

    let render_arena = RenderArena::new(&dom);
    println!("{:?}", render_arena);
}
