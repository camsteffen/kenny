pub(crate) trait VecExt<T> {
    /// Appends a default element to the back of a collection
    /// and returns a mutable reference to the value.
    fn push_default_mut(&mut self) -> &mut T
    where
        T: Default;

    /// Appends an element to the back of a collection
    /// and returns a mutable reference to the value.
    fn push_mut(&mut self, value: T) -> &mut T;

    fn remove_indices(&mut self, indices: &[usize]);

    fn remove_indices_copy(&mut self, indices: &[usize])
    where
        T: Copy;

    fn retain_indexed(&mut self, f: impl FnMut(usize, &mut T) -> bool);

    fn retain_indices(&mut self, indices: &[usize]);

    fn retain_indices_copy(&mut self, indices: &[usize])
    where
        T: Copy;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_default_mut(&mut self) -> &mut T
    where
        T: Default,
    {
        self.push_mut(Default::default())
    }

    fn push_mut(&mut self, value: T) -> &mut T {
        self.push(value);
        self.last_mut().unwrap()
    }

    fn remove_indices(&mut self, indices: &[usize]) {
        if indices.is_empty() {
            return;
        }
        let len = self.len();
        let v = &mut **self;
        let mut iter = indices[1..].iter().peekable();
        let mut del = 1;
        for i in (indices[0] + 1)..len {
            if iter.peek() == Some(&&i) {
                del += 1;
                iter.next();
            } else {
                v.swap(i - del, i);
            }
        }
        assert_eq!(iter.next(), None);
        self.truncate(len - indices.len());
    }

    fn remove_indices_copy(&mut self, indices: &[usize])
    where
        T: Copy,
    {
        if indices.is_empty() {
            return;
        }
        let len = self.len();
        let v = &mut **self;
        let mut iter = indices[1..].iter().peekable();
        let mut del = 1;
        for i in (indices[0] + 1)..len {
            if iter.peek() == Some(&&i) {
                del += 1;
                iter.next();
            } else {
                v[i - del] = v[i];
            }
        }
        assert_eq!(iter.next(), None);
        self.truncate(len - indices.len());
    }

    fn retain_indexed(&mut self, mut f: impl FnMut(usize, &mut T) -> bool) {
        let mut kept = 0;
        let v = &mut **self;
        let len = v.len();
        for i in 0..len {
            if f(i, &mut v[i]) {
                if kept < i {
                    v.swap(kept, i);
                }
                kept += 1;
            }
        }
        if kept < len {
            self.truncate(kept);
        }
    }

    fn retain_indices(&mut self, indices: &[usize]) {
        let v = &mut **self;
        for (i, j) in indices
            .iter()
            .copied()
            .enumerate()
            .skip_while(|&(i, j)| i == j)
        {
            v.swap(i, j);
        }
        if indices.len() < self.len() {
            self.truncate(indices.len());
        }
    }

    fn retain_indices_copy(&mut self, indices: &[usize])
    where
        T: Copy,
    {
        let v = &mut **self;
        for (i, j) in indices
            .iter()
            .copied()
            .enumerate()
            .skip_while(|&(i, j)| i == j)
        {
            v[i] = v[j];
        }
        if indices.len() < self.len() {
            self.truncate(indices.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::vec_ext::VecExt;

    #[test]
    fn test_retain_indices() {
        fn test(values: &[usize], indices: &[usize], expected: &[usize]) {
            let mut v = Vec::from(values);
            v.retain_indices(indices);
            assert_eq!(v, Vec::from(expected));

            let mut v = Vec::from(values);
            v.retain_indices_copy(indices);
            assert_eq!(v, Vec::from(expected));
        }
        test(&[1, 2, 3], &[], &[]);
        test(&[1, 2, 3], &[0], &[1]);
        test(&[1, 2, 3], &[1], &[2]);
        test(&[1, 2, 3], &[0, 1], &[1, 2]);
        test(&[1, 2, 3], &[1, 2], &[2, 3]);
        test(&[1, 2, 3], &[0, 2], &[1, 3]);
        test(&[1, 2, 3], &[0, 1, 2], &[1, 2, 3]);
        test(&[1, 2, 3, 4], &[0, 2, 3], &[1, 3, 4]);
    }

    #[test]
    fn test_remove_indices() {
        fn test(values: &[usize], indices: &[usize], expected: &[usize]) {
            let mut v = Vec::from(values);
            v.remove_indices(indices);
            assert_eq!(v, Vec::from(expected));
        }
        test(&[1, 2, 3], &[], &[1, 2, 3]);
        test(&[1, 2, 3], &[0], &[2, 3]);
        test(&[1, 2, 3], &[1], &[1, 3]);
        test(&[1, 2, 3], &[0, 1], &[3]);
        test(&[1, 2, 3], &[1, 2], &[1]);
        test(&[1, 2, 3], &[0, 2], &[2]);
        test(&[1, 2, 3], &[0, 1, 2], &[]);
        test(&[1, 2, 3, 4], &[0, 2, 3], &[2]);
    }
}
