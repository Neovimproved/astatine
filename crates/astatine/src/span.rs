use std::{fmt::Debug, slice};

#[derive(Debug)]
pub struct Span<T> {
    items: Vec<T>,
    position: usize,
}

impl<T> Span<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items, position: 0 }
    }

    pub fn pos(&self) -> usize {
        self.position
    }
}

impl<T> Span<T> {
    pub fn peek(&self) -> Option<&T> {
        self.items.get(self.position)
    }

    pub fn peek_nth(&self, idx: usize) -> Option<&T> {
        self.items.get(self.position + idx)
    }

    pub fn next(&mut self) -> Option<&T> {
        let Some(item) = self.items.get(self.position) else {
            return None;
        };

        self.position += 1;
        Some(item)
    }

    pub fn skip(&mut self, count: usize) -> Option<&T> {
        self.position = (self.position + count).min(self.items.len());
        self.items.get(self.position)
    }
}

impl<'a, T> IntoIterator for &'a Span<T> {
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items[self.position..].into_iter()
    }
}

/// y and x, line and column
pub fn to_coordinates(s: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;

    for (idx, ch) in s.chars().enumerate() {
        if idx == pos {
            break;
        }

        if ch == '\n' {
            line += 1;
            line_start = idx + 1;
        }
    }

    (line, pos - line_start)
}
