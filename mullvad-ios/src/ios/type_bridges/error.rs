#[derive(uniffi::Object)]
pub struct AnyError {
    inner: Box<dyn core::error::Error + Send + Sync>,
}
#[uniffi::export]
impl AnyError {
    pub fn error(&self) -> String {
        format!("{}", self.inner)
    }

    pub fn error_debug(&self) -> String {
        format!("{:?}", self.inner)
    }

    #[uniffi::constructor]
    pub fn message(text: String) -> Self {
        Self {
            inner: Box::new(StringError { inner: text }),
        }
    }
}

impl<E: core::error::Error + Send + Sync + 'static> From<E> for AnyError {
    fn from(value: E) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

impl std::fmt::Debug for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}
impl std::fmt::Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

/// Internal struct used to allow Strings as errors
struct StringError {
    inner: String,
}
impl std::fmt::Debug for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}
impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl core::error::Error for StringError {}
