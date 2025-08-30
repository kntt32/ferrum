use super::dom::*;
use super::tokenizer::*;

pub struct TreeConstructor {
    tokenizer: Tokenizer,
    dom: DomArena,
    stack_of_open_elements: Vec<usize>,
    insertion_mode: InsertionMode,
    mode_flag: bool,
    context_element: Option<DomNodeIdx>,
}

impl TreeConstructor {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            dom: DomArena::new(),
            stack_of_open_elements: vec![DomArena::DOCUMENT_IDX],
            insertion_mode: InsertionMode::INIT,
            mode_flag: false,
            context_element: None,
        }
    }

    fn current_node(&self) -> DomNodeIdx {
        self.stack_of_open_elements.len() - 1
    }

    fn adjusted_current_node(&self) -> DomNodeIdx {
        if let Some(context_element) = self.context_element
            && self.stack_of_open_elements.len() == 1
        {
            context_element
        } else {
            self.current_node()
        }
    }

    pub fn handle_token(&mut self, token: Token) {
        assert!(!self.stack_of_open_elements.is_empty());

        let adjusted_current_node = self.dom.get(self.adjusted_current_node());
        if adjusted_current_node.namespace() == Namespace::Html || token == Token::Eof {
            match self.insertion_mode {
                InsertionMode::Initial => self.handle_token_init(token),
                _ => todo!(),
            }
        } else {
            todo!();
        }
    }

    pub fn handle_token_init(&mut self, token: Token) {
        match token {
            Token::Character(c)
                if ['\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}', '\u{0020}'].contains(&c) =>
            {
                ()
            }
            Token::Comment(comment) => todo!(),
            _ => todo!(),
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

impl InsertionMode {
    pub const INIT: Self = Self::Initial;
}
