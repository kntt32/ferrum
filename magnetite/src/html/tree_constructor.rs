use super::dom::*;
use super::tokenizer::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TreeConstructor {
    insertion_mode: InsertionMode,
    original_insertion_mode: Option<InsertionMode>,
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
    is_fragment: bool,
    position: Option<DomNodeIdx>,
    errors: Vec<ParseError>,
}

impl TreeConstructor {
    pub fn new() -> Self {
        Self {
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: None,
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
            is_fragment: false,
            position: None,
            errors: Vec::new(),
        }
    }

    pub fn dom(&self) -> &DomArena {
        &self.arena
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
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

    pub fn handle_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn switch_to(&mut self, insertion_mode: InsertionMode) {
        if [InsertionMode::Text, InsertionMode::InTableText].contains(&insertion_mode) {
            self.original_insertion_mode = Some(self.insertion_mode);
        }
        self.insertion_mode = insertion_mode;
    }

    fn switch_to_original_insertion_mode(&mut self) {
        let original_insertion_mode = self.original_insertion_mode.take().unwrap();
        self.switch_to(original_insertion_mode);
    }

    pub fn adjusted_current_node_namespace(&self) -> Namespace {
        self.arena[self.adjusted_current_node()].namespace()
    }

    pub fn handle_token(&mut self, token: Token) -> Option<TokenizerState> {
        match self.insertion_mode {
            InsertionMode::Initial => self.handle_token_initial(token),
            InsertionMode::BeforeHtml => self.handle_token_before_html(token),
            InsertionMode::BeforeHead => self.handle_token_before_head(token),
            InsertionMode::InHead => self.handle_token_in_head(token),
            InsertionMode::AfterHead => self.handle_token_after_head(token),
            InsertionMode::InBody => self.handle_token_in_body(token),
            InsertionMode::AfterBody => self.handle_token_after_body(token),
            InsertionMode::AfterAfterBody => self.handle_token_after_after_token(token),
            InsertionMode::Text => self.handle_token_text(token),
            mode => unimplemented!("{:?}", mode),
        }
    }

    fn handle_token_text(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Character(c) => {
                self.insert_character(c);
            }
            Token::Eof => {
                self.error(ParseError::EofInText);
            }
            Token::EndTag { name, .. } if &name == "script" => {
                unimplemented!();
            }
            Token::EndTag { .. } => {
                self.open_elements.pop();
                self.switch_to_original_insertion_mode();
            }
            _ => (),
        }
        None
    }

    fn handle_token_after_after_token(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }

            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::Eof => (),
            _ => {
                self.error(ParseError::UnexpectedTokenInAfterAfterBody);
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
        }
        None
    }

    fn handle_token_after_body(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                self.error(ParseError::UnexpectedDoctype);
            }
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
            Token::EndTag { ref name, .. } if name == "html" => {
                if self.is_fragment {
                    self.error(ParseError::HtmlEndTagInFragmentParse);
                } else {
                    self.switch_to(InsertionMode::AfterAfterBody);
                }
            }
            Token::Eof => (),
            _ => {
                self.error(ParseError::UnexpectedEndTag);
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
        }
        None
    }

    fn handle_token_initial(&mut self, token: Token) -> Option<TokenizerState> {
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
        None
    }

    fn opened_element(&self, name: &str) -> bool {
        for i in 1..self.open_elements.len() {
            let idx = self.open_elements[i];
            if let DomNodeType::Element { name: ref n, .. } = self.arena[idx].node_type
                && n == name
            {
                return true;
            }
        }
        false
    }

    fn close_element(&mut self, name: &str) {
        for i in (1..self.open_elements.len()).rev() {
            let idx = self.open_elements[i];
            if let DomNodeType::Element { name: ref n, .. } = self.arena[idx].node_type
                && n == name
            {
                self.open_elements.resize_with(i, || panic!());
                break;
            }
        }
    }

    fn close_element_until(&mut self, close: &[&str], until: &str) {
        for i in (1..self.open_elements.len()).rev() {
            let idx = self.open_elements[i];
            if let DomNodeType::Element { name: ref n, .. } = self.arena[idx].node_type {
                if n == until {
                    break;
                }
                if close.contains(&n.as_str()) {
                    self.open_elements.remove(i);
                }
            }
        }
    }

    fn has_an_element_in_scope(&self, name: &str) -> bool {
        const SCOPE: &[&str] = &[
            "applet", "caption", "html", "table", "td", "th", "marquee", "object", "select",
            "template",
        ];
        for idx in &self.open_elements {
            if let DomNodeType::Element { name: ref n, .. } = self.arena[*idx].node_type {
                if n == name {
                    return true;
                }
                if SCOPE.contains(&n.as_str()) {
                    return false;
                }
            }
        }
        false
    }

    fn has_an_element_in_button_scope(&self, name: &str) -> bool {
        if self.has_an_element_in_scope(name) {
            true
        } else {
            for idx in &self.open_elements {
                if let DomNodeType::Element { name: ref n, .. } = self.arena[*idx].node_type {
                    if n == name && self.arena[*idx].namespace() == Namespace::Html {
                        return true;
                    }
                    if n == "button" {
                        return false;
                    }
                }
            }
            false
        }
    }

    fn handle_token_in_body(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Character('\0') => self.error(ParseError::UnexpectedNullCharacter),
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.reconstruct_the_active_formatting_elements();
                self.insert_character(c);
            }
            Token::Character(c) => {
                self.reconstruct_the_active_formatting_elements();
                self.insert_character(c);
                self.frameset_ok = false;
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => self.error(ParseError::UnexpectedDoctype),
            Token::StartTag { name, attributes } if &name == "html" => {
                self.error(ParseError::UnexpectedStartTag);
                if let Some(html_idx) = self.arena.get_child_element(DomArena::DOCUMENT_IDX, "html")
                {
                    if let DomNodeType::Element {
                        attributes: ref mut real_attributes,
                        ..
                    } = self.arena[html_idx].node_type
                    {
                        for attribute in attributes {
                            if !real_attributes.contains_key(&attribute.0) {
                                real_attributes.insert(attribute.0.clone(), attribute.1.clone());
                            }
                        }
                    }
                }
            }
            Token::StartTag { ref name, .. }
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&name.as_str()) =>
            {
                self.switch_to(InsertionMode::InHead);
                self.handle_token(token);
            }
            Token::EndTag { ref name, .. } if name == "template" => {
                self.switch_to(InsertionMode::InHead);
                self.handle_token(token);
            }
            Token::StartTag { ref name, .. } if ["body", "frameset"].contains(&name.as_str()) => {
                unimplemented!();
            }
            Token::Eof => {
                if !self.template_insertion_modes.is_empty() {
                    self.switch_to(InsertionMode::InTemplate);
                    self.handle_token(token);
                } else {
                    for i in 1..self.open_elements.len() {
                        let node_idx = self.open_elements[i];
                        if let DomNodeType::Element { ref name, .. } =
                            self.arena[node_idx].node_type
                            && ![
                                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt",
                                "rtc", "tbody", "td", "tfoot", "th", "thead", "tr", "body", "html",
                            ]
                            .contains(&name.as_str())
                        {
                            self.error(ParseError::UnclosedElementAtEof);
                            break;
                        }
                    }
                }
            }
            Token::EndTag { ref name, .. } if name == "body" => {
                if !self.opened_element("html") {
                    self.error(ParseError::UnclosedElement);
                } else {
                    for i in 1..self.open_elements.len() {
                        let node_idx = self.open_elements[i];
                        if let DomNodeType::Element { ref name, .. } =
                            self.arena[node_idx].node_type
                            && ![
                                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt",
                                "rtc", "tbody", "td", "tfoot", "th", "thead", "tr", "body", "html",
                            ]
                            .contains(&name.as_str())
                        {
                            self.error(ParseError::UnclosedElement);
                            break;
                        }
                    }

                    self.switch_to(InsertionMode::AfterBody);
                }
            }
            Token::StartTag { name, attributes }
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&name.as_str()) =>
            {
                if self.opened_element("p") {
                    self.close_element("p");
                }
                if let DomNodeType::Element { ref name, .. } =
                    self.arena[self.current_node()].node_type
                    && ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&name.as_str())
                {
                    self.open_elements.pop();
                }
                self.insert_element(name, attributes);
            }
            Token::StartTag { name, attributes } => {
                self.reconstruct_the_active_formatting_elements();
                self.insert_element(name, attributes);
            }
            Token::EndTag { name }
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&name.as_str()) =>
            {
                if !self.opened_element(&name) {
                    self.error(ParseError::UnexpectedEndTag);
                } else {
                    self.close_element_until(
                        &[
                            "dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc",
                        ],
                        "h1",
                    );
                    let current_node = self.current_node();
                    let DomNodeType::Element { name: ref n, .. } =
                        self.arena[current_node].node_type
                    else {
                        panic!();
                    };
                    if &name != n {
                        self.error(ParseError::UnexpectedEndTag);
                    }
                    self.close_element(&name);
                }
            }
            Token::EndTag { name } if &name == "p" => {
                if !self.has_an_element_in_button_scope("p") {
                    self.error(ParseError::ElementNotFoundInButtonScope);
                    self.insert_element("p".to_string(), HashMap::new());
                }
                self.close_element("p");
            }
            Token::EndTag { name } => {
                if !self.opened_element("html") {
                    self.error(ParseError::UnexpectedEndTag);
                } else {
                    self.close_element(&name);
                }
            }
        }
        None
    }

    fn reconstruct_the_active_formatting_elements(&mut self) {
        if self.active_formatting_elements.is_empty() {
            return;
        }

        let Some(Some(entry)) = self.active_formatting_elements.last() else {
            return;
        };
        if self.open_elements.contains(entry) {
            return;
        }

        unimplemented!("{:?}", entry);
    }

    fn handle_token_after_head(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.insert_character(c);
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
                self.insert_element(name, attributes);
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
                self.insert_element("body".to_string(), HashMap::new());
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
            }
        }
        None
    }

    fn set_original_insertion_mode(&mut self) {
        self.original_insertion_mode = Some(self.insertion_mode);
    }

    fn parse_generic_raw_text_element(&mut self, name: String, attributes: HashMap<String, String>) -> Option<TokenizerState> {
        self.insert_element(name, attributes);
        self.set_original_insertion_mode();
        self.switch_to(InsertionMode::Text);
        
        Some(TokenizerState::RawText)
    }

    fn handle_token_in_head(&mut self, token: Token) -> Option<TokenizerState> {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                self.insert_character(c);
                None
            }
            Token::Comment(text) => {
                self.insert_comment(text);
                None
            }
            Token::Doctype { .. } => {
                self.error(ParseError::UnexpectedDoctype);
                None
            },
            Token::StartTag { ref name, .. } if name == "html" => {
                self.switch_to(InsertionMode::InBody);
                self.handle_token(token);
                None
            }
            Token::StartTag {
                name, attributes, ..
            } if ["noframes", "style"].contains(&name.as_str()) => {
                self.parse_generic_raw_text_element(name, attributes)
            }
            Token::StartTag {
                name, attributes, ..
            } if ["base", "basefont", "bgsound", "link"].contains(&name.as_str()) => {
                self.insert_element(name, attributes);
                None
            }
            Token::StartTag { name, .. }
                if [
                    "meta", "title", "noscript", "noframes", "style", "script", "head", "template",
                ]
                .contains(&name.as_str()) =>
            {
                unimplemented!("{:?}", name);
            }
            Token::StartTag { name, .. } if &name == "head" => {
                self.error(ParseError::UnexpectedHeadTag);
                None
            }
            Token::EndTag { ref name, .. } if name == "head" => {
                self.open_elements.pop();
                self.switch_to(InsertionMode::AfterHead);
                None
            }
            Token::EndTag { .. } => {
                self.error(ParseError::UnexpectedEndTag);
                None
            },
            _ => {
                self.open_elements.pop();
                self.switch_to(InsertionMode::AfterHead);
                self.handle_token(token);
                None
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

    fn insert_character(&mut self, c: char) {
        self.insert(
            DomNode::new(
                DomNodeType::Character(c),
                self.adjusted_current_node_namespace(),
            ),
            false,
        );
    }

    fn insert_element(&mut self, name: String, attributes: HashMap<String, String>) -> DomNodeIdx {
        self.insert(
            DomNode::new(
                DomNodeType::Element { name, attributes },
                self.adjusted_current_node_namespace(),
            ),
            false,
        )
    }

    fn insert_element_with_only_add_to_element_stack(
        &mut self,
        name: String,
        attributes: HashMap<String, String>,
    ) -> DomNodeIdx {
        self.insert(
            DomNode::new(
                DomNodeType::Element { name, attributes },
                self.adjusted_current_node_namespace(),
            ),
            true,
        )
    }

    fn insert(&mut self, node: DomNode, only_add_to_element_stack: bool) -> DomNodeIdx {
        let is_element = if let DomNodeType::Element { .. } = node.node_type {
            true
        } else {
            false
        };
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node();
        let nodeidx = if !only_add_to_element_stack {
            self.arena.append_child(adjusted_insertion_location, node)
        } else {
            self.arena.push(node)
        };
        if is_element {
            self.open_elements.push(nodeidx);
        }
        nodeidx
    }

    fn handle_token_before_head(&mut self, token: Token) -> Option<TokenizerState> {
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
                let head_idx = self.insert_element(name, attributes);
                self.head_element = Some(head_idx);
                self.switch_to(InsertionMode::InHead);
            }
            Token::EndTag { name, .. }
                if !["head", "body", "html", "br"].contains(&name.as_str()) =>
            {
                self.error(ParseError::UnexpectedEndTag);
            }
            _ => {
                let head_idx = self.insert_element("head".to_string(), HashMap::new());
                self.head_element = Some(head_idx);
                self.switch_to(InsertionMode::InHead);
                self.handle_token(token);
            }
        }
        None
    }

    fn handle_token_before_html(&mut self, token: Token) -> Option<TokenizerState> {
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
                self.insert_element(name, attributes);
                self.switch_to(InsertionMode::BeforeHead);
            }
            Token::EndTag { name } if !["head", "body", "html", "br"].contains(&name.as_str()) => {
                self.error(ParseError::UnexpectedEndTag);
            }
            _ => {
                self.insert_element("html".to_string(), HashMap::new());
                self.switch_to(InsertionMode::BeforeHead);
                self.handle_token(token);
            }
        }
        None
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
