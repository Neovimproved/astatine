use core::slice;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Span<I: Iterator>
where
    I::Item: Debug,
{
    iter: I,
    position: usize,
}

impl<I: Iterator> Span<I>
where
    I::Item: Debug,
{
    pub fn new(iter: I) -> Self {
        Self { iter, position: 0 }
    }

    pub fn pos(&self) -> usize {
        self.position
    }
}

impl<'a, T> Span<slice::Iter<'a, T>>
where
    T: Debug,
{
    pub fn peek(&self) -> Option<&T> {
        self.iter.as_slice().first()
    }

    #[allow(dead_code)]
    pub fn peek_nth(&self, idx: usize) -> Option<&T> {
        self.iter.as_slice().get(idx)
    }

    pub fn next(&mut self) -> Option<&T> {
        self.position += 1;
        self.iter.next()
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
