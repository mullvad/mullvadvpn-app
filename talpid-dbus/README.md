# talpid-dbus
Communicate with different system components over Dbus.

## zbus
Enable the `zbus` feature to use [`zbus`](https://github.com/z-galaxy/zbus) as the Dbus backend. This is experimental and inomplete, but functional!
The following table list the system components that `talpid-dbus` can communicate with using `zbus` instead of [`dbus-rs`](https://github.com/diwic/dbus-rs).

| System component | `zbus` |
| ---------------- | ------ |
| NetworkManager   |   ❌   |
| systemd          |   ✅   |
| systemd-resolved |   ✅   |
