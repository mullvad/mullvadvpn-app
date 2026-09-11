/// An `anyhow::Error` like error, but tailored for uniffi.
/// To get the error message in your host language (Swift in our case), you can
/// do something like the following:
///
/// ```swift
/// do {
///     try throwingFunction()
/// } catch let error as ErasedError {
///     print("error: \(error.message())")
/// }
/// ```
#[derive(uniffi::Object)]
pub struct ErasedError {
    inner: Box<dyn std::error::Error + Send + Sync>,
}
struct StringError {
    msg: String,
}
impl std::fmt::Debug for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.msg, f)
    }
}
impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.msg, f)
    }
}
impl std::error::Error for StringError {}

#[uniffi::export]
impl ErasedError {
    #[uniffi::constructor]
    pub fn msg(value: String) -> Self {
        Self::from(StringError { msg: value })
    }

    pub fn as_string(&self) -> String {
        self.to_string()
    }
}
impl std::fmt::Debug for ErasedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}
impl std::fmt::Display for ErasedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl<E: std::error::Error + Send + Sync + 'static> From<E> for ErasedError {
    fn from(value: E) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}
