/// Drop guard that executes the provided callback function when dropped.
pub struct OnDrop<F = Box<dyn FnOnce() + Send>>
where
    F: FnOnce() + Send,
{
    callback: Option<F>,
}

impl<F: FnOnce() + Send> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            callback();
        }
    }
}

impl<F: FnOnce() + Send> OnDrop<F> {
    pub fn new(callback: F) -> Self {
        Self {
            callback: Some(callback),
        }
    }
}
