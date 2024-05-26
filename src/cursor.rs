pub trait Cursor<T> {
    fn cursor(&self) -> usize;
    fn at_cursor(&self) -> &T;
    fn increase(&mut self);
    fn decrease(&mut self);
}

pub struct VecCursor<T> {
    cursor: usize,
    list: Vec<T>,
}

impl<T> Cursor<T> for VecCursor<T> {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn at_cursor(&self) -> &T {
        &self.list[self.cursor]
    }

    fn increase(&mut self) {
        self.cursor = (self.cursor + 1) % self.list.len();
    }

    fn decrease(&mut self) {
        if self.list.is_empty() {
            self.cursor = 0;
        } else if self.cursor == 0 {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }
}

impl<T> VecCursor<T> {
    pub fn substitute<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.list.clear();
        self.list.extend(iter);

        if self.list.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.list.len() {
            self.cursor = self.list.len() - 1;
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.list[..]
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.list[..]
    }
}

impl<T> FromIterator<T> for VecCursor<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let list = Vec::from_iter(iter);
        Self { cursor: 0, list }
    }
}
