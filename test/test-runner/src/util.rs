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

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error {
    inner: InnerError,
}

#[cfg(unix)]
#[derive(thiserror::Error, Debug)]
enum InnerError {
    #[error("Failed to get the specified user")]
    GetUser(#[source] nix::Error),
    #[error("The specified user was not found")]
    MissingUser,
    #[error("Failed to set uid")]
    SetUid(#[source] nix::Error),
    #[error("Failed to set gid")]
    SetGid(#[source] nix::Error),
}

#[cfg(target_os = "windows")]
#[derive(thiserror::Error, Debug)]
enum InnerError {}

impl From<InnerError> for Error {
    fn from(inner: InnerError) -> Self {
        Self { inner }
    }
}

#[cfg(target_os = "windows")]
pub fn as_unprivileged_user<T>(
    unpriv_user: &str,
    func: impl FnOnce() -> T,
) -> Result<T, nix::Error> {
    // NOTE: no-op
    let _ = unpriv_user;
    Ok(func())
}

#[cfg(unix)]
pub fn as_unprivileged_user<T>(unpriv_user: &str, func: impl FnOnce() -> T) -> Result<T, Error> {
    let original_uid = nix::unistd::getuid();
    let original_gid = nix::unistd::getgid();

    let user = nix::unistd::User::from_name(unpriv_user)
        .map_err(InnerError::GetUser)?
        .ok_or(InnerError::MissingUser)?;
    let uid = user.uid;
    let gid = user.gid;

    nix::unistd::setegid(gid).map_err(InnerError::SetGid)?;
    let restore_gid = OnDrop::new(|| {
        if let Err(error) = nix::unistd::setegid(original_gid) {
            log::error!("Failed to restore gid: {error}");
        }
    });

    nix::unistd::seteuid(uid).map_err(InnerError::SetUid)?;
    let restore_uid = OnDrop::new(|| {
        if let Err(error) = nix::unistd::seteuid(original_uid) {
            log::error!("Failed to restore uid: {error}");
        }
    });

    let result = Ok(func());

    drop(restore_uid);
    drop(restore_gid);

    result
}
