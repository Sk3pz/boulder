use crate::CodePos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputReader {
    stream: Vec<char>,
    pos: CodePos,
}

impl InputReader {
    pub fn new<S: Into<String>>(file: Option<String>, input: S) -> Self {
        let pos = if file.is_some() {
            CodePos::new(file.unwrap(), 1, 1)
        } else {
            CodePos::default()
        };
        Self {
            stream: input.into().chars().collect(),
            pos,
        }
    }

    pub fn pos(&self) -> CodePos {
        self.pos.clone()
    }

    pub fn consume(&mut self) -> Option<char> {
        if self.stream.is_empty() {
            return None;
        }

        let ch = self.stream.remove(0);
        self.pos.next();

        if ch == '\n' {
            self.pos.newline();
        }

        Some(ch)
    }

    pub fn peek_at(&self, n: usize) -> Option<char> {
        self.stream.get(n).cloned()
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_at(0)
    }

}