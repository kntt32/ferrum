use super::input_stream_preprocessor::InputStreamPreprocessor;

type State = TokenizerState;

#[derive(Debug)]
pub struct Tokenizer {
    state: State,
    return_state: Option<State>,
    string: String,
    string_index: usize,
    temporary_buffer: String,
    temporary_token: Option<Token>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
}

impl Tokenizer {
    pub fn new(preprocessor: InputStreamPreprocessor) -> Self {
        Self {
            state: State::INIT,
            return_state: None,
            string: preprocessor.preprocess(),
            string_index: 0,
            temporary_buffer: String::new(),
            temporary_token: None,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn emit(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn flush(&mut self) {
        for c in self.temporary_buffer.chars() {
            self.tokens.push(Token::Character(c));
        }
        self.temporary_buffer.clear();
    }

    fn look(&self) -> Option<char> {
        self.string[self.string_index..].chars().next()
    }

    fn read(&mut self) -> Option<char> {
        let c = self.look()?;
        self.string_index += c.len_utf8();
        Some(c)
    }

    fn unread(&mut self, c: Option<char>) {
        c.map(|c| {
            self.string_index -= c.len_utf8();
        });
    }

    fn switch_to(&mut self, state: State) {
        self.state = state;

        match state {
            State::CharacterReference => {
                self.temporary_buffer.clear();
                self.temporary_buffer.push('&');
            }
            _ => (),
        }
    }

    fn set_return_state(&mut self, state: State) {
        self.return_state = Some(state);
    }

    fn return_state(&mut self) {
        if let Some(state) = self.return_state.take() {
            self.switch_to(state);
        }
    }

    pub fn step(&mut self) {
        match self.state {
            State::Data => self.step_data(),
            State::CharacterReference => self.step_character_reference(),
            State::TagOpen => self.step_tag_open(),
            State::TagName => self.step_tag_name(),
            _ => todo!(),
        }
    }

    fn step_data(&mut self) {
        assert_eq!(self.state, State::Data);

        match self.read() {
            Some('&') => {
                self.set_return_state(State::Data);
                self.switch_to(State::CharacterReference);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.emit(Token::Character('\0'));
            }
            Some('<') => self.switch_to(State::TagOpen),
            Some(c) => self.emit(Token::Character(c)),
            None => self.emit(Token::Eof),
        }
    }

    fn step_tag_open(&mut self) {
        match self.read() {
            Some('!') => self.switch_to(State::MarkupDeclarationOpen),
            Some('/') => self.switch_to(State::EndTagOpen),
            Some(c) if c.is_ascii_alphabetic() => {
                self.temporary_token = Some(Token::StartTag {
                    name: String::new(),
                });
                self.unread(Some(c));
                self.switch_to(State::TagName);
            }
            Some('?') => {
                self.error(ParseError::UnexpectedQuestionMarkInsteadOfTagName);
                self.temporary_token = Some(Token::Comment(String::new()));
                self.unread(Some('?'));
                self.switch_to(State::BogusComment);
            }
            None => {
                self.error(ParseError::EofBeforeTagName);
                self.emit(Token::Character('<'));
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.error(ParseError::InvalidFirstCharacterOfTagName);
                self.emit(Token::Character('<'));
                self.unread(Some(c));
                self.switch_to(State::Data);
            }
        }
    }

    fn step_tag_name(&mut self) {
        match self.read() {
            Some('\u{0009}') | Some('\u{000a}') | Some('\u{000c}') | Some('\u{0020}') => {
                self.switch_to(State::BeforeAttributeName)
            }
            Some('\u{002f}') => self.switch_to(State::SelfClosingStartTag),
            Some('\u{003e}') => {
                let token = self.temporary_token.take().unwrap();
                self.emit(token);
                self.switch_to(State::Data);
            }
            Some(mut c) if c.is_ascii_uppercase() => {
                let Some(Token::StartTag { ref mut name }) = self.temporary_token else {
                    panic!();
                };
                c.make_ascii_lowercase();
                name.push(c);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                let Some(Token::StartTag { ref mut name }) = self.temporary_token else {
                    panic!();
                };
                name.push('\u{fffd}');
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
            }
            Some(c) => {
                let Some(Token::StartTag { ref mut name }) = self.temporary_token else {
                    panic!();
                };
                name.push(c);
            }
        }
    }

    fn step_character_reference(&mut self) {
        assert_eq!(self.state, State::CharacterReference);

        match self.read() {
            Some(c) if c.is_ascii_alphanumeric() => {
                self.unread(Some(c));
                self.switch_to(State::NamedCharacterReference);
            }
            Some('#') => {
                self.temporary_buffer.push('#');
                self.switch_to(State::NumericCharacterReference);
            }
            c => {
                self.unread(c);
                self.return_state();
                self.flush();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    EofBeforeTagName,
    InvalidFirstCharacterOfTagName,
    EofInTag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenizerState {
    Data,
    RcData,
    RawText,
    ScriptData,
    PlainText,
    TagOpen,
    EndTagOpen,
    TagName,
    RcDataLessThanSign,
    RcDataEndTagOpen,
    RcDataEndTagName,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValue,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    CDataSection,
    CDataSectionBracket,
    CDataSectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
}

impl TokenizerState {
    pub const INIT: Self = Self::Data;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Doctype,
    StartTag { name: String },
    EndTag,
    Comment(String),
    Character(char),
    Eof,
}
