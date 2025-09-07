use ab_glyph::FontRef;
use std::sync::LazyLock;

static DEFAULT_FONT: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    FontRef::try_from_slice(include_bytes!("../../../assets/fonts/NotoSansJP-VariableFont_wght.ttf")).unwrap()
});

#[derive(Clone, Debug)]
pub struct Font {

}

