use super::input_stream_preprocessor::InputStreamPreprocessor;

type State = TokenizerState;

pub struct Tokenizer {
    state: State,
    return_state: Option<State>,
    string: String,
    string_index: usize,
    temporary_buffer: String,
    temporary_token: Option<Token>,
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
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn flush(&mut self, token_notify: &mut impl FnMut(Token)) {
        for c in self.temporary_buffer.chars() {
            token_notify(Token::Character(c));
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

    fn look_str(&self, len: usize) -> Option<&str> {
        self.string[self.string_index..].get(..len)
    }

    fn read_str(&mut self, len: usize) -> Option<&str> {
        let s = self.string[self.string_index..].get(..len)?;
        self.string_index += len;
        Some(s)
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

    pub fn step(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        error_notify: &mut impl FnMut(ParseError),
    ) {
        match self.state {
            State::Data => self.step_data(token_notify, error_notify),
            State::CharacterReference => self.step_character_reference(token_notify, error_notify),
            State::TagOpen => self.step_tag_open(token_notify, error_notify),
            State::TagName => self.step_tag_name(token_notify, error_notify),
            State::EndTagOpen => self.step_end_tag_open(token_notify, error_notify),
            State::MarkupDeclarationOpen => {
                self.step_markup_declaration_open(token_notify, error_notify)
            }
            State::Doctype => todo!(),
            _ => unimplemented!("{:?}", self.state),
        }
    }

    fn step_markup_declaration_open(
        &mut self,
        _token_notify: &mut impl FnMut(Token),
        _error_notify: &mut impl FnMut(ParseError),
    ) {
        assert_eq!(self.state, State::MarkupDeclarationOpen);
        /*
        const DOCTYPE: &str = "DOCTYPE";
        const CDATA: &str = "[CDATA[";

        if self.look_str(2) == Some("--") {
            self.read_str(2);
            self.temporary_token = Some(Token::Comment(String::new()));
            self.switch_to(State::CommentStart);
        }else if let Some(s) = self.look_str(DOCTYPE.len()) && s.eq_ignore_asci_case(DOCTYPE) {
            self.read_str(DOCTYPE.len());
            self.switch_to(State::Doctype);
        }else if let Some(s) = self.look_str(CDATA.len()) && s.eq_ignore_asci_case(CDATA) {
            self.read_str(CDATA.len());
            self.
        }*/
        todo!();
    }

    fn step_data(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        error_notify: &mut impl FnMut(ParseError),
    ) {
        assert_eq!(self.state, State::Data);

        match self.read() {
            Some('&') => {
                self.set_return_state(State::Data);
                self.switch_to(State::CharacterReference);
            }
            Some('\0') => {
                error_notify(ParseError::UnexpectedNullCharacter);
                token_notify(Token::Character('\0'));
            }
            Some('<') => self.switch_to(State::TagOpen),
            Some(c) => token_notify(Token::Character(c)),
            None => token_notify(Token::Eof),
        }
    }

    fn step_tag_open(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        error_notify: &mut impl FnMut(ParseError),
    ) {
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
                error_notify(ParseError::UnexpectedQuestionMarkInsteadOfTagName);
                self.temporary_token = Some(Token::Comment(String::new()));
                self.unread(Some('?'));
                self.switch_to(State::BogusComment);
            }
            None => {
                error_notify(ParseError::EofBeforeTagName);
                token_notify(Token::Character('<'));
                token_notify(Token::Eof);
            }
            Some(c) => {
                error_notify(ParseError::InvalidFirstCharacterOfTagName);
                token_notify(Token::Character('<'));
                self.unread(Some(c));
                self.switch_to(State::Data);
            }
        }
    }

    fn step_end_tag_open(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        error_notify: &mut impl FnMut(ParseError),
    ) {
        match self.read() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.temporary_token = Some(Token::EndTag {
                    name: String::new(),
                });
                self.unread(Some(c));
                self.switch_to(State::TagName);
            }
            Some('>') => {
                error_notify(ParseError::MissingEndTagName);
                self.switch_to(State::Data);
            }
            None => {
                error_notify(ParseError::EofBeforeTagName);
                token_notify(Token::Character('<'));
                token_notify(Token::Character('/'));
                token_notify(Token::Eof);
            }
            Some(c) => {
                error_notify(ParseError::InvalidFirstCharacterOfTagName);
                self.temporary_token = Some(Token::Comment(String::new()));
                self.unread(Some(c));
                self.switch_to(State::BogusComment);
            }
        }
    }

    fn step_tag_name(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        error_notify: &mut impl FnMut(ParseError),
    ) {
        match self.read() {
            Some('\u{0009}') | Some('\u{000a}') | Some('\u{000c}') | Some('\u{0020}') => {
                self.switch_to(State::BeforeAttributeName)
            }
            Some('\u{002f}') => self.switch_to(State::SelfClosingStartTag),
            Some('\u{003e}') => {
                let token = self.temporary_token.take().unwrap();
                token_notify(token);
                self.switch_to(State::Data);
            }
            Some('\0') => {
                error_notify(ParseError::UnexpectedNullCharacter);
                let name = match self.temporary_token {
                    Some(Token::StartTag { ref mut name }) => name,
                    Some(Token::EndTag { ref mut name }) => name,
                    _ => panic!(),
                };
                name.push('\u{fffd}');
            }
            Some(mut c) => {
                let name = match self.temporary_token {
                    Some(Token::StartTag { ref mut name }) => name,
                    Some(Token::EndTag { ref mut name }) => name,
                    _ => panic!(),
                };
                if c.is_ascii_uppercase() {
                    c.make_ascii_lowercase();
                }
                name.push(c);
            }
            None => {
                error_notify(ParseError::EofInTag);
                token_notify(Token::Eof);
            }
        }
    }

    fn step_character_reference(
        &mut self,
        token_notify: &mut impl FnMut(Token),
        _error_notify: &mut impl FnMut(ParseError),
    ) {
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
                self.flush(token_notify);
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
    MissingEndTagName,
    ExpectedDoctypeButGotSomethingElse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenizerState {
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
    Doctype(String),
    StartTag { name: String },
    EndTag { name: String },
    Comment(String),
    Character(char),
    Eof,
}
