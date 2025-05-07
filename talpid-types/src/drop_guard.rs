/// Create a value that executes function `F` when dropped.
#[must_use = "Should be assigned to a variable and implicitly or explicitly dropped"]
pub fn on_drop<F: FnOnce()>(f: F) -> OnDrop<F> {
    OnDrop(Some(f))
}

/// A type that executes a function when dropped.
pub struct OnDrop<F: FnOnce() = Box<dyn FnOnce()>>(Option<F>);

impl<F: FnOnce() + 'static> OnDrop<F> {
    /// Box the inner function and erase its type.
    pub fn boxed(mut self) -> OnDrop {
        let f = self.0.take();
        let f = f.map(|f| Box::new(f) as Box<_>);
        OnDrop(f)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f()
        }
    }
}
