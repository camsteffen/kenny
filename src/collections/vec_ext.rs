pub(crate) trait VecExt<T> {
    /// Appends a default element to the back of a collection
    /// and returns a mutable reference to the value.
    fn push_default_mut(&mut self) -> &mut T
    where
        T: Default;

    /// Appends an element to the back of a collection
    /// and returns a mutable reference to the value.
    fn push_mut(&mut self, value: T) -> &mut T;
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
}
