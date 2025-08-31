use super::dom::*;
use super::tokenizer::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TreeConstructor {
    insertion_mode: InsertionMode,
    template_insertion_modes: Vec<InsertionMode>,
    open_elements: Vec<DomNodeIdx>,
    active_formatting_elements: Vec<Option<DomNodeIdx>>,
    head_element: Option<DomNodeIdx>,
    form_element: Option<DomNodeIdx>,
    quirks_mode: QuirksMode,
    pending_table_characters: String,
    arena: DomArena,
    document: DomNodeIdx,
    saw_doctype: bool,
    frameset_ok: bool,
    position: Option<DomNodeIdx>,
    errors: Vec<ParseError>,
}

impl TreeConstructor {
    pub fn new() -> Self {
        Self {
            insertion_mode: InsertionMode::Initial,
            template_insertion_modes: Vec::new(),
            open_elements: vec![DomArena::DOCUMENT_IDX],
            active_formatting_elements: Vec::new(),
            head_element: None,
            form_element: None,
            quirks_mode: QuirksMode::NoQuirks,
            pending_table_characters: String::new(),
            arena: DomArena::new(),
            document: DomArena::DOCUMENT_IDX,
            saw_doctype: false,
            frameset_ok: true,
            position: None,
            errors: Vec::new(),
        }
    }

    pub fn mode(&self) -> InsertionMode {
        self.insertion_mode
    }

    fn current_node(&self) -> DomNodeIdx {
        *self.open_elements.last().unwrap()
    }

    fn adjusted_current_node(&self) -> DomNodeIdx {
        self.current_node()
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn switch_to(&mut self, insertion_mode: InsertionMode) {
        self.insertion_mode = insertion_mode;
    }

    pub fn adjusted_current_node_namespace(&self) -> Namespace {
        self.arena[self.adjusted_current_node()].namespace()
    }

    pub fn handle_token(&mut self, token: Token) {
        match self.insertion_mode {
            InsertionMode::Initial => self.handle_token_initial(token),
            InsertionMode::BeforeHtml => self.handle_token_before_html(token),
            InsertionMode::BeforeHead => self.handle_token_before_head(token),
            InsertionMode::InHead => self.handle_token_in_head(token),
            InsertionMode::AfterHead => self.handle_token_after_head(token),
            InsertionMode::InBody => self.handle_token_in_body(token),
            mode => unimplemented!("{:?}", mode),
        }
    }

    fn handle_token_initial(&mut self, token: Token) {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                ()
            }
            Token::Doctype { name, .. } => {
                self.saw_doctype = true;

                self.quirks_mode = if let Some(name) = &name
                    && name.eq_ignore_ascii_case("html")
                {
                    QuirksMode::NoQuirks
                } else {
                    QuirksMode::Quirks
                };
                self.switch_to(InsertionMode::BeforeHtml);
            }
            Token::Comment(text) => {
                let node = DomNode::new(DomNodeType::Comment(text), Namespace::Html);
                self.arena.append_child(self.document, node);
            }
            _ => {
                self.error(ParseError::ExpectedDoctypeButGotSomethingElse);
                self.quirks_mode = QuirksMode::Quirks;
                self.switch_to(InsertionMode::BeforeHtml);
                self.handle_token(token);
            }
        }
    }

    fn handle_token_in_body(&mut self, _token: Token) {
        todo!()
    }

    fn handle_token_after_head(&mut self, token: Token) {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.insert_element(
                    DomNode::new(
                        DomNodeType::Character(c),
                        self.adjusted_current_node_namespace(),
                    ),
                    false,
                );
            }
            Token::Comment(text) => self.insert_comment(text),
            Token::Doctype { .. } => self.error(ParseError::UnexpectedDoctype),
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::StartTag {
                name, attributes, ..
            } if name == "body" => {
                self.insert_element(
                    DomNode::new(
                        DomNodeType::Element { name, attributes },
                        self.adjusted_current_node_namespace(),
                    ),
                    false,
                );
                self.frameset_ok = false;
                self.switch_to(InsertionMode::InBody);
            }
            Token::StartTag { ref name, .. }
                if [
                    "frameset", "base", "basefont", "bgsound", "link", "meta", "noframes",
                    "script", "style", "template", "title",
                ]
                .contains(&name.as_str()) =>
            {
                unimplemented!("{:?}", name);
            }
            Token::EndTag { ref name, .. } if name == "template" => unimplemented!("{:?}", name),
            Token::StartTag { ref name, .. } if name == "head" => {
                self.error(ParseError::UnexpectedHeadTag);
            }
            Token::EndTag { ref name, .. } if !["body", "html", "br"].contains(&name.as_str()) => {
                self.error(ParseError::UnexpectedEndTag);
            }
            _ => {
                self.insert_element(
                    DomNode::new(
                        DomNodeType::Element {
                            name: "body".to_string(),
                            attributes: HashMap::new(),
                        },
                        Namespace::Html,
                    ),
                    false,
                );
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
        }
    }

    fn handle_token_in_head(&mut self, token: Token) {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.insert_element(
                    DomNode::new(DomNodeType::Character(c), Namespace::Html),
                    false,
                );
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => self.error(ParseError::UnexpectedDoctype),
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::StartTag {
                name, attributes, ..
            } if ["base", "basefont", "bgsound", "link"].contains(&name.as_str()) => {
                self.insert_element(
                    DomNode::new(DomNodeType::Element { name, attributes }, Namespace::Html),
                    false,
                );
            }
            Token::StartTag { name, .. }
                if [
                    "meta", "title", "noscript", "noframes", "style", "script", "head", "template",
                ]
                .contains(&name.as_str()) =>
            {
                unimplemented!("{:?}", name)
            }
            Token::StartTag { name, .. } if &name == "head" => {
                self.error(ParseError::UnexpectedHeadTag);
            }
            Token::EndTag { .. } => self.error(ParseError::UnexpectedEndTag),
            _ => {
                self.open_elements.pop();
                self.switch_to(InsertionMode::AfterHead);
                self.handle_token(token);
            }
        }
    }

    fn appropriate_place_for_inserting_a_node(&self) -> DomNodeIdx {
        self.current_node()
    }

    fn insert_position(&self) -> DomNodeIdx {
        if let Some(position) = self.position {
            position
        } else {
            self.appropriate_place_for_inserting_a_node()
        }
    }

    fn insert_comment(&mut self, text: String) {
        let insert_position = self.insert_position();
        let domnode = DomNode::new(
            DomNodeType::Comment(text),
            self.arena[insert_position].namespace(),
        );
        self.arena.append_child(insert_position, domnode);
    }

    fn insert_element(&mut self, node: DomNode, only_add_to_element_stack: bool) -> DomNodeIdx {
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node();
        let nodeidx = if !only_add_to_element_stack {
            self.arena.append_child(adjusted_insertion_location, node)
        } else {
            self.arena.push(node)
        };
        self.open_elements.push(nodeidx);
        nodeidx
    }

    fn handle_token_before_head(&mut self, token: Token) {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                ()
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => self.error(ParseError::UnexpectedDoctype),
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::StartTag { name, attributes } if name == "head" => {
                let head_idx = self.insert_element(
                    DomNode::new(DomNodeType::Element { name, attributes }, Namespace::Html),
                    false,
                );
                self.head_element = Some(head_idx);
                self.switch_to(InsertionMode::InHead);
            }
            Token::EndTag { name, .. }
                if !["head", "body", "html", "br"].contains(&name.as_str()) =>
            {
                self.error(ParseError::UnexpectedEndTag);
            }
            _ => {
                let head_idx = self.insert_element(
                    DomNode::new(
                        DomNodeType::Element {
                            name: "head".to_string(),
                            attributes: HashMap::new(),
                        },
                        Namespace::Html,
                    ),
                    false,
                );
                self.head_element = Some(head_idx);
                self.switch_to(InsertionMode::InHead);
                self.handle_token(token);
            }
        }
    }

    fn handle_token_before_html(&mut self, token: Token) {
        match token {
            Token::Doctype { .. } => self.error(ParseError::UnexpectedDoctype),
            Token::Comment(text) => {
                let node = DomNode::new(DomNodeType::Comment(text), Namespace::Html);
                self.arena.append_child(self.document, node);
            }
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                ()
            }
            Token::StartTag { name, attributes } if &name == "html" => {
                let html_node =
                    DomNode::new(DomNodeType::Element { name, attributes }, Namespace::Html);
                let html_id = self.arena.append_child(DomArena::DOCUMENT_IDX, html_node);
                self.open_elements.push(html_id);
                self.switch_to(InsertionMode::BeforeHead);
            }
            Token::EndTag { name } if !["head", "body", "html", "br"].contains(&name.as_str()) => {
                self.error(ParseError::UnexpectedEndTag);
            }
            _ => {
                let html_node = DomNode::new(
                    DomNodeType::Element {
                        name: "html".to_string(),
                        attributes: HashMap::new(),
                    },
                    Namespace::Html,
                );
                let html_id = self.arena.append_child(DomArena::DOCUMENT_IDX, html_node);
                self.open_elements.push(html_id);
                self.switch_to(InsertionMode::BeforeHead);
                self.handle_token(token);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug, Clone, Copy)]
pub enum QuirksMode {
    NoQuirks,
    LimitedQuirks,
    Quirks,
}
