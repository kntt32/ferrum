use super::dom::*;
use super::tokenizer::*;
use std::collections::HashMap;

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
            errors: Vec::new(),
        }
    }

    fn current_node(&self) -> DomNodeIdx {
        *self.open_elements.last().unwrap()
    }

    fn adjusted_current_node(&self) -> DomNodeIdx {
        *self.open_elements.last().unwrap()
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
            _ => unimplemented!(),
        }
    }

    pub fn handle_token_initial(&mut self, token: Token) {
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
    Inhead,
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
