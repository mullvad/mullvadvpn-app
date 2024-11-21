pub use super::unix::*;

pub fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
    use nix::net::if_::if_nametoindex;
    use std::num::NonZero;

    log::info!("Binding socket to {interface:?}");

    let interface_index = if_nametoindex(interface)
        .map_err(eyre::Report::from)
        .and_then(|code| NonZero::new(code).ok_or_eyre("Non-zero error code"))
        .wrap_err("Failed to get interface index")?;

    socket.bind_device_by_index_v4(Some(interface_index))?;
    Ok(())
}
