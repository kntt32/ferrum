use html::byte_stream_decoder::ByteStreamDecoder;
use html::input_stream_preprocessor::InputStreamPreprocessor;
use html::tokenizer::Tokenizer;
use html::tree_constructor::*;
use magnetite::arena::*;
use magnetite::css::CssomArena;
use magnetite::css::Origin;
use magnetite::css::Parser;
use magnetite::css::Token as CssToken;
use magnetite::css::Tokenizer as CssTokenizer;
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
.my_class {
    color: blue;
    font-size: 10px;
}
h1.my_class {
    color: red;
    font-size: 20px;
}
        "#;

    let tokenizer = CssTokenizer::new(s);
    let mut parser = Parser::new(tokenizer);
    let stylesheet = parser.parse_a_style_sheet();
    println!("{:?}", stylesheet);
    let mut cssom = CssomArena::new();
    cssom.add_stylesheet(&stylesheet, Origin::UserAgent);
    println!("{:?}", cssom);
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
                font-size: 20px;
            }
        </style>
    </head>
    <body>
        <h1>
            Hello
        </h1>
        <p>World!</p>
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

    let dom = tree_constructor.take_dom();
    println!("{:?}", dom);
    println!("{:?}", dom.style());

    let render_arena = RenderArena::new(&dom);
    println!("{:?}", render_arena);

    let cssom = dom.cssom();
    println!("{:?}", cssom);
}
