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

#[cfg(target_os = "windows")]
pub fn as_unprivileged<T>(unpriv_user: &str, func: impl FnOnce() -> T) -> Result<T, nix::Error> {
    // NOTE: no-op
    let _ = unpriv_user;
    Ok(func())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get the specified user")]
    GetUser(#[source] nix::Error),
    #[error("The specified user was not found")]
    MissingUser,
}

#[cfg(unix)]
pub fn as_unprivileged<T>(unpriv_user: &str, func: impl FnOnce() -> T) -> Result<T, Error> {
    let original_uid = nix::unistd::getuid();
    let original_gid = nix::unistd::getgid();

    let user = nix::unistd::User::from_name(unpriv_user)
        .map_err(Error::GetUser)?
        .ok_or(Error::MissingUser)?;
    let uid = user.uid;
    let gid = user.gid;

    if let Err(error) = nix::unistd::setegid(gid) {
        log::error!("Failed to set gid: {error}");
    }
    if let Err(error) = nix::unistd::seteuid(uid) {
        log::error!("Failed to set uid: {error}");
    }

    let func_result = func();

    if let Err(error) = nix::unistd::seteuid(original_uid) {
        log::error!("Failed to restore uid: {error}");
    }
    if let Err(error) = nix::unistd::setegid(original_gid) {
        log::error!("Failed to restore gid: {error}");
    }

    Ok(func_result)
}
