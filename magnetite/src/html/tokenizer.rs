use super::dom::Namespace;
use super::input_stream_preprocessor::InputStreamPreprocessor;
use super::tree_constructor::TreeConstructor;
use std::collections::HashMap;
use std::mem;

type State = TokenizerState;

pub struct Tokenizer<'a> {
    state: State,
    return_state: Option<State>,
    string: String,
    string_index: usize,
    temporary_buffer: String,
    temporary_token: Option<Token>,
    tree_constructor: &'a mut TreeConstructor,
    appropriate_end_tag_name: Option<String>,
    current_attribute: Option<(String, String)>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(
        preprocessor: InputStreamPreprocessor,
        tree_constructor: &'a mut TreeConstructor,
    ) -> Self {
        Self {
            state: State::INIT,
            return_state: None,
            string: preprocessor.preprocess(),
            string_index: 0,
            temporary_buffer: String::new(),
            temporary_token: None,
            tree_constructor,
            appropriate_end_tag_name: None,
            current_attribute: None,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn emit(&mut self, token: Token) {
        if let Token::StartTag { ref name, .. } = token {
            self.appropriate_end_tag_name = Some(name.clone());
        }
        if let Some(state) = self.tree_constructor.handle_token(token) {
            self.switch_to(state);
        }
    }

    fn emit_temporary_token(&mut self) {
        let token = self.temporary_token.take().unwrap();
        self.emit(token);
    }

    fn error(&mut self, error: ParseError) {
        self.tree_constructor.handle_error(error);
    }

    fn adjusted_current_node_namespace(&self) -> Namespace {
        self.tree_constructor.adjusted_current_node_namespace()
    }

    fn flush(&mut self) {
        let temporary_buffer = mem::take(&mut self.temporary_buffer);

        for c in temporary_buffer.chars() {
            self.emit(Token::Character(c));
        }
        self.temporary_buffer = temporary_buffer;
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
        if let Some(c) = c {
            self.string_index -= c.len_utf8();
        }
    }

    fn look_str(&self, len: usize) -> Option<&str> {
        self.string[self.string_index..].get(..len)
    }

    fn read_str(&mut self, len: usize) -> Option<&str> {
        let s = self.string[self.string_index..].get(..len)?;
        self.string_index += len;
        Some(s)
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

    pub fn step(&mut self) -> Option<()> {
        if self.look().is_none() {
            self.emit(Token::Eof);
            return None;
        }

        match self.state {
            State::Data => self.step_data(),
            State::CharacterReference => self.step_character_reference(),
            State::TagOpen => self.step_tag_open(),
            State::TagName => self.step_tag_name(),
            State::EndTagOpen => self.step_end_tag_open(),
            State::MarkupDeclarationOpen => self.step_markup_declaration_open(),
            State::Doctype => self.step_doctype(),
            State::BeforeDoctypeName => self.step_before_doctype_name(),
            State::DoctypeName => self.step_doctype_name(),
            State::RawText => self.step_raw_text(),
            State::RawTextLessThanSign => self.step_raw_text_less_than(),
            State::RawTextEndTagOpen => self.step_raw_text_end_tag_open(),
            State::RawTextEndTagName => self.step_raw_text_end_tag_name(),
            State::RcData => self.step_rcdata(),
            State::RcDataLessThanSign => self.step_rcdata_less_than_sign(),
            State::RcDataEndTagOpen => self.step_rcdata_end_tag_open(),
            State::RcDataEndTagName => self.step_rcdata_end_tag_name(),
            State::BeforeAttributeName => self.step_before_attribute_name(),
            State::AttributeName => self.step_attribute_name(),
            State::BeforeAttributeValue => self.step_before_attribute_value(),
            State::AttributeValueDoubleQuoted => self.step_attribute_value_double_quoted(),
            State::AttributeValueSingleQuoted => self.step_attribute_value_single_quoted(),
            State::AttributeValueUnquoted => self.step_attribute_value_unquoted(),
            State::AfterAttributeValueQuoted => self.step_after_attribute_value_quoted(),
            _ => unimplemented!("{:?}\n{:?}", self.state, &self.string[self.string_index..]),
        }

        Some(())
    }

    fn step_after_attribute_value_quoted(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {
                self.switch_to(State::BeforeAttributeName);
            }
            Some('/') => {
                self.switch_to(State::SelfClosingStartTag);
            }
            Some('>') => {
                self.switch_to(State::Data);
                if let Some(current_attribute) = self.current_attribute.take() {
                    let Some(Token::StartTag {
                        ref mut attributes, ..
                    }) = self.temporary_token
                    else {
                        panic!();
                    };
                    attributes.insert(current_attribute.0, current_attribute.1);
                }
                let temporary_token = self.temporary_token.take().unwrap();
                self.emit(temporary_token);
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
            }
            c => {
                self.error(ParseError::MissingWhitespaceBetweenAttributes);
                self.unread(c);
                self.switch_to(State::BeforeAttributeName);
            }
        }
    }

    fn step_attribute_value_single_quoted(&mut self) {
        match self.read() {
            Some('\'') => {
                self.switch_to(State::AfterAttributeValueQuoted);
            }
            Some('&') => {
                self.set_return_state(State::AttributeValueDoubleQuoted);
                self.switch_to(State::CharacterReference);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.current_attribute.as_mut().unwrap().1.push('\u{fffd}');
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.current_attribute.as_mut().unwrap().1.push(c);
            }
        }
    }

    fn step_attribute_value_double_quoted(&mut self) {
        match self.read() {
            Some('"') => {
                self.switch_to(State::AfterAttributeValueQuoted);
            }
            Some('&') => {
                self.set_return_state(State::AttributeValueDoubleQuoted);
                self.switch_to(State::CharacterReference);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.current_attribute.as_mut().unwrap().1.push('\u{fffd}');
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.current_attribute.as_mut().unwrap().1.push(c);
            }
        }
    }

    fn step_attribute_value_unquoted(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {
                self.switch_to(State::BeforeAttributeName);
            }
            Some('&') => {
                self.set_return_state(State::AttributeValueUnquoted);
                self.switch_to(State::CharacterReference);
            }
            Some('>') => {
                self.switch_to(State::Data);
                if let Some(current_attribute) = self.current_attribute.take() {
                    let Some(Token::StartTag {
                        ref mut attributes, ..
                    }) = self.temporary_token
                    else {
                        panic!();
                    };
                    attributes.insert(current_attribute.0, current_attribute.1);
                }
                let temporary_token = self.temporary_token.take().unwrap();
                self.emit(temporary_token);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.current_attribute.as_mut().unwrap().1.push('\u{fffd}');
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
            }
            Some(c) => {
                if ['"', '\'', '<', '=', '`'].contains(&c) {
                    self.error(ParseError::UnexpectedCharacterInUnquotedAttributeValue);
                }
                self.current_attribute.as_mut().unwrap().1.push(c);
            }
        }
    }

    fn step_before_attribute_value(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {}
            Some('"') => {
                self.switch_to(State::AttributeValueDoubleQuoted);
            }
            Some('\'') => {
                self.switch_to(State::AttributeValueSingleQuoted);
            }
            Some('>') => {
                self.error(ParseError::MissingAttributeValue);
                self.switch_to(State::Data);
            }
            c => {
                self.unread(c);
                self.switch_to(State::AttributeValueUnquoted);
            }
        }
    }

    fn step_attribute_name(&mut self) {
        match self.read() {
            Some(c)
                if [
                    '\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}', '\u{002f}', '\u{003e}',
                ]
                .contains(&c) =>
            {
                self.unread(Some(c));
                self.switch_to(State::AfterAttributeName);
            }
            None => {
                self.unread(None);
                self.switch_to(State::AfterAttributeName);
            }
            Some('=') => {
                self.switch_to(State::BeforeAttributeValue);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.current_attribute.as_mut().unwrap().0.push('\u{fffd}');
            }
            Some(mut c) => {
                if ['"', '\'', '<'].contains(&c) {
                    self.error(ParseError::UnexpectedCharacterInAttributeName);
                }
                c.make_ascii_lowercase();
                self.current_attribute.as_mut().unwrap().0.push(c);
            }
        }
    }

    fn step_before_attribute_name(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {}
            c if [Some('/'), Some('>'), None].contains(&c) => {
                self.unread(c);
                self.switch_to(State::AfterAttributeName);
            }
            Some('=') => {
                self.error(ParseError::UnexpectedEqualsSignBeforeAttributeName);
                todo!()
            }
            c => {
                if let Some(current_attribute) = self.current_attribute.take() {
                    let Some(Token::StartTag { attributes, .. }) = &mut self.temporary_token else {
                        panic!();
                    };
                    attributes.insert(current_attribute.0, current_attribute.1);
                }
                self.current_attribute = Some((String::new(), String::new()));
                self.unread(c);
                self.switch_to(State::AttributeName);
            }
        }
    }

    fn step_rcdata_end_tag_name(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', ' ', '/', '>'].contains(&c) => {
                let Token::EndTag { name, .. } = self.temporary_token.as_ref().unwrap() else {
                    panic!();
                };
                if Some(name) == self.appropriate_end_tag_name.as_ref() {
                    match c {
                        '/' => {
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        '>' => {
                            let token = self.temporary_token.take().unwrap();
                            self.emit(token);
                            self.switch_to(State::Data);
                        }
                        _ => {
                            self.switch_to(State::BeforeAttributeName);
                        }
                    }
                } else {
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('/'));
                    self.flush();
                    self.unread(Some(c));
                    self.switch_to(State::RcData);
                }
            }
            Some(mut c) if c.is_ascii_alphabetic() => {
                self.temporary_buffer.push(c);
                c.make_ascii_lowercase();
                let Some(Token::EndTag { ref mut name }) = self.temporary_token else {
                    panic!();
                };
                name.push(c);
            }
            c => {
                self.emit(Token::Character('<'));
                self.emit(Token::Character('/'));
                self.flush();
                self.unread(c);
                self.switch_to(State::RcData);
            }
        }
    }

    fn step_rcdata_end_tag_open(&mut self) {
        match self.read() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.temporary_token = Some(Token::EndTag {
                    name: String::new(),
                });
                self.unread(Some(c));
                self.switch_to(State::RcDataEndTagName);
            }
            c => {
                self.emit(Token::Character('<'));
                self.emit(Token::Character('/'));
                self.unread(c);
                self.switch_to(State::RcData);
            }
        }
    }

    fn step_rcdata_less_than_sign(&mut self) {
        match self.read() {
            Some('/') => {
                self.temporary_buffer.clear();
                self.switch_to(State::RcDataEndTagOpen);
            }
            c => {
                self.emit(Token::Character('<'));
                self.unread(c);
                self.switch_to(State::RcData);
            }
        }
    }

    fn step_rcdata(&mut self) {
        match self.read() {
            Some('&') => {
                self.set_return_state(State::RcData);
                self.switch_to(State::CharacterReference);
            }
            Some('<') => {
                self.switch_to(State::RcDataLessThanSign);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.emit(Token::Character('\u{fffd}'));
            }
            None => {
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.emit(Token::Character(c));
            }
        }
    }

    fn step_raw_text_end_tag_name(&mut self) {
        match self.read() {
            Some(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c)
                    && Some(&self.temporary_buffer) == self.appropriate_end_tag_name.as_ref() =>
            {
                self.switch_to(State::BeforeAttributeName);
            }
            Some('/') if Some(&self.temporary_buffer) == self.appropriate_end_tag_name.as_ref() => {
                self.switch_to(State::SelfClosingStartTag);
            }
            Some('>') if Some(&self.temporary_buffer) == self.appropriate_end_tag_name.as_ref() => {
                self.switch_to(State::Data);
                let name = mem::take(&mut self.temporary_buffer);
                self.emit(Token::EndTag { name });
                self.appropriate_end_tag_name = None;
                self.switch_to(State::Data);
            }
            Some(mut c) if c.is_ascii_alphabetic() => {
                c.make_ascii_lowercase();
                self.temporary_buffer.push(c);
            }
            c => {
                self.emit(Token::Character('<'));
                self.emit(Token::Character('/'));
                self.flush();
                self.unread(c);
                self.switch_to(State::RawText);
            }
        }
    }

    fn step_raw_text_end_tag_open(&mut self) {
        match self.read() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.temporary_token = Some(Token::EndTag {
                    name: String::new(),
                });
                self.unread(Some(c));
                self.switch_to(State::RawTextEndTagName);
            }
            c => {
                self.emit(Token::Character('<'));
                self.emit(Token::Character('/'));
                self.unread(c);
                self.switch_to(State::RawText);
            }
        }
    }

    fn step_raw_text_less_than(&mut self) {
        match self.read() {
            Some('/') => {
                self.temporary_buffer.clear();
                self.switch_to(State::RawTextEndTagOpen);
            }
            c => {
                self.emit(Token::Character('<'));
                self.unread(c);
                self.switch_to(State::RawText);
            }
        }
    }

    fn step_raw_text(&mut self) {
        match self.read() {
            Some('<') => {
                self.switch_to(State::RawTextLessThanSign);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.emit(Token::Character('\u{fffd}'));
            }
            None => {
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.emit(Token::Character(c));
            }
        }
    }

    fn step_doctype_name(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {
                self.switch_to(State::AfterDoctypeName)
            }
            Some('\u{003e}') => {
                self.switch_to(State::Data);
                self.emit_temporary_token();
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                let Some(Token::Doctype { ref mut name, .. }) = self.temporary_token else {
                    panic!();
                };
                if name.is_none() {
                    *name = Some(String::new());
                }
                name.as_mut().unwrap().push('\u{fffd}');
            }
            None => {
                self.error(ParseError::EofInDoctype);
                let Some(Token::Doctype {
                    ref mut force_quirks,
                    ..
                }) = self.temporary_token
                else {
                    panic!();
                };
                *force_quirks = true;
                self.emit_temporary_token();
                self.emit(Token::Eof);
            }
            Some(mut c) => {
                c.make_ascii_lowercase();
                let Some(Token::Doctype { ref mut name, .. }) = self.temporary_token else {
                    panic!();
                };
                if name.is_none() {
                    *name = Some(String::new());
                }
                name.as_mut().unwrap().push(c);
            }
        }
    }

    fn step_before_doctype_name(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => (),
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                self.temporary_token = Some(Token::Doctype {
                    name: Some("\u{fffd}".to_string()),
                    public_id: None,
                    system_id: None,
                    force_quirks: false,
                });
                self.switch_to(State::DoctypeName);
            }
            Some('>') => {
                self.error(ParseError::MissingDoctypeName);
                self.emit(Token::Doctype {
                    name: Some(String::new()),
                    public_id: None,
                    system_id: None,
                    force_quirks: true,
                });
                self.switch_to(State::Data);
            }
            None => {
                self.error(ParseError::EofInDoctype);
                self.emit(Token::Doctype {
                    name: Some(String::new()),
                    public_id: None,
                    system_id: None,
                    force_quirks: true,
                });
                self.emit(Token::Eof);
            }
            Some(mut c) => {
                c.make_ascii_lowercase();
                self.temporary_token = Some(Token::Doctype {
                    name: Some(format!("{}", c)),
                    public_id: None,
                    system_id: None,
                    force_quirks: false,
                });
                self.switch_to(State::DoctypeName);
            }
        }
    }

    fn step_doctype(&mut self) {
        match self.read() {
            Some(c) if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{0020}'].contains(&c) => {
                self.switch_to(State::BeforeDoctypeName)
            }
            Some('>') => {
                self.unread(Some('>'));
                self.switch_to(State::BeforeDoctypeName);
            }
            None => {
                self.error(ParseError::EofInDoctype);
                self.emit(Token::Doctype {
                    name: None,
                    public_id: None,
                    system_id: None,
                    force_quirks: true,
                });
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.error(ParseError::MissingWhiteSpaceBeforeDoctypeName);
                self.unread(Some(c));
                self.switch_to(State::BeforeDoctypeName);
            }
        }
    }

    fn step_markup_declaration_open(&mut self) {
        assert_eq!(self.state, State::MarkupDeclarationOpen);

        const TWO_HYPHEN: &str = "--";
        const DOCTYPE: &str = "DOCTYPE";
        const CDATA: &str = "[CDATA[";
        if self.look_str(TWO_HYPHEN.len()) == Some(TWO_HYPHEN) {
            self.read_str(TWO_HYPHEN.len());
            self.temporary_token = Some(Token::Comment(String::new()));
            self.switch_to(State::CommentStart);
        } else if let Some(s) = self.look_str(DOCTYPE.len())
            && s.eq_ignore_ascii_case(DOCTYPE)
        {
            self.read_str(DOCTYPE.len());
            self.switch_to(State::Doctype);
        } else if self.look_str(CDATA.len()) == Some(CDATA) {
            self.read_str(CDATA.len());
            if self.adjusted_current_node_namespace() == Namespace::Html {
                self.switch_to(State::CDataSection);
            } else {
                self.error(ParseError::CDataInHtmlContent);
                self.temporary_token = Some(Token::Comment(CDATA.to_string()));
                self.switch_to(State::BogusComment);
            }
        } else {
            self.error(ParseError::IncorrectlyOpenedComment);
            self.temporary_token = Some(Token::Comment(String::new()));
            self.switch_to(State::BogusComment);
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
                self.temporary_token = Some(Token::new_start_tag());
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

    fn step_end_tag_open(&mut self) {
        match self.read() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.temporary_token = Some(Token::EndTag {
                    name: String::new(),
                });
                self.unread(Some(c));
                self.switch_to(State::TagName);
            }
            Some('>') => {
                self.error(ParseError::MissingEndTagName);
                self.switch_to(State::Data);
            }
            None => {
                self.error(ParseError::EofBeforeTagName);
                self.emit(Token::Character('<'));
                self.emit(Token::Character('/'));
                self.emit(Token::Eof);
            }
            Some(c) => {
                self.error(ParseError::InvalidFirstCharacterOfTagName);
                self.temporary_token = Some(Token::Comment(String::new()));
                self.unread(Some(c));
                self.switch_to(State::BogusComment);
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
                self.switch_to(State::Data);
                self.emit(token);
            }
            Some('\0') => {
                self.error(ParseError::UnexpectedNullCharacter);
                let name = match self.temporary_token {
                    Some(Token::StartTag { ref mut name, .. }) => name,
                    Some(Token::EndTag { ref mut name }) => name,
                    _ => panic!(),
                };
                name.push('\u{fffd}');
            }
            Some(mut c) => {
                let name = match self.temporary_token {
                    Some(Token::StartTag { ref mut name, .. }) => name,
                    Some(Token::EndTag { ref mut name }) => name,
                    _ => panic!(),
                };
                if c.is_ascii_uppercase() {
                    c.make_ascii_lowercase();
                }
                name.push(c);
            }
            None => {
                self.error(ParseError::EofInTag);
                self.emit(Token::Eof);
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
    MissingEndTagName,
    ExpectedDoctypeButGotSomethingElse,
    UnexpectedDoctype,
    UnexpectedEndTag,
    CDataInHtmlContent,
    IncorrectlyOpenedComment,
    EofInDoctype,
    EofInText,
    MissingWhiteSpaceBeforeDoctypeName,
    MissingDoctypeName,
    UnexpectedHeadTag,
    UnclosedElementAtEof,
    UnclosedElement,
    UnexpectedStartTag,
    ElementNotFoundInButtonScope,
    HtmlEndTagInFragmentParse,
    UnexpectedTokenInAfterAfterBody,
    UnexpectedEqualsSignBeforeAttributeName,
    MissingWhitespaceBetweenAttributes,
    UnexpectedCharacterInUnquotedAttributeValue,
    MissingAttributeValue,
    UnexpectedCharacterInAttributeValue,
    UnexpectedCharacterInAttributeName,
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
    AttributeValueUnquoted,
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
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Character(char),
    Eof,
}

impl Token {
    pub fn new_start_tag() -> Self {
        Self::StartTag {
            name: String::new(),
            attributes: HashMap::new(),
        }
    }
}
