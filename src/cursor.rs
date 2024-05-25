pub struct Cursor<T> {
    cursor: usize,
    list: Vec<T>,
}

impl<T> Cursor<T> {
    pub fn substitute<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.list.clear();
        self.list.extend(iter);

        if self.list.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.list.len() {
            self.cursor = self.list.len() - 1;
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn at_cursor(&self) -> &T {
        &self.list[self.cursor]
    }

    pub fn increase(&mut self) {
        self.cursor = (self.cursor + 1) % self.list.len();
    }

    pub fn decrease(&mut self) {
        if self.list.is_empty() {
            self.cursor = 0;
        } else if self.cursor == 0 {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.list[..]
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.list[..]
    }
}

impl<T> FromIterator<T> for Cursor<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let list = Vec::from_iter(iter);
        Self { cursor: 0, list }
    }
}
