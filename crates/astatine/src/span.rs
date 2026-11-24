use std::{
    collections::{VecDeque, vec_deque},
    fmt::Debug,
};

#[derive(Debug)]
pub struct Span<T> {
    items: VecDeque<T>,
    position: usize,
}

impl<T> Span<T> {
    pub fn new(items: VecDeque<T>) -> Self {
        Self { items, position: 0 }
    }

    pub fn pos(&self) -> usize {
        self.position
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn peek_nth(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    pub fn next(&mut self) -> Option<T> {
        let item = self.items.pop_front()?;
        self.position += 1;
        Some(item)
    }
}

impl<T> IntoIterator for Span<T> {
    type Item = T;

    type IntoIter = vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
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
