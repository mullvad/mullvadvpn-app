use serde::Deserialize;
use zbus::zvariant::OwnedObjectPath;

#[zbus::proxy(
    interface = "org.freedesktop.portable1.Manager",
    default_service = "org.freedesktop.portable1",
    default_path = "/org/freedesktop/portable1"
)]

pub trait PortabledD {
    // TODO: return type is object-path
    async fn get_image(&self, name: &str) -> zbus::Result<OwnedObjectPath>;
    async fn list_images(&self) -> zbus::Result<Vec<ImageListing>>;

    /// Attach a portable service image.
    ///
    /// - `image` refers to a path or a name.
    /// - if `runtime=true`, the image will not persist across reboots.
    /// - `copy_mode` is one of `""`, `"copy"`, `"symlink"` `"mixed"`.
    ///
    /// ## dbus type signature:
    /// ```systemd
    /// AttachImage(in  s image,
    ///  in  as matches,
    ///  in  s profile,
    ///  in  b runtime,
    ///  in  s copy_mode,
    ///  out a(sss) changes);
    /// ```
    #[zbus(allow_interactive_auth)]
    async fn attach_image(
        &self,
        image: &str,
        matches: &[&str],
        profile: &str,
        runtime: bool,
        copy_mode: &str,
    ) -> zbus::Result<Vec<Change>>;

    /// Detach a portable service image.
    ///
    /// - `image` refers to a path or a name.
    /// - `runtime` indicates whether the image was attached only for the current boot session.
    ///
    /// ## dbus type signature:
    /// ```systemd
    /// AttachImage(in  s image,
    ///  in  as matches,
    ///  in  s profile,
    ///  in  b runtime,
    ///  in  s copy_mode,
    ///  out a(sss) changes);
    /// ```
    #[zbus(allow_interactive_auth)]
    async fn detach_image(&self, image: &str, runtime: bool) -> zbus::Result<Vec<Change>>;
}

/// A change that was applied to the system as a result of `PortableD::attach_image`.
#[derive(Clone, Debug, Deserialize, zbus::zvariant::Type)]
pub struct Change {
    pub change_type: String,
    pub path: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, zbus::zvariant::Type)]
pub struct ImageListing {
    // TODO: field names
    pub name: String,
    pub image_type: String,
    pub read_only: bool,
    pub creation_time: u64,
    pub modification_time: u64,
    pub current_disk_space: u64,
    pub usage: String,
    pub object_path: OwnedObjectPath, // TODO: type is object-path
}
