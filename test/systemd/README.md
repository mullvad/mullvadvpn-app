# systemd service

This directory contains an example systemd timer for running the tests nightly.

To install a user service for the current user, just edit the `.timer` and `.service` files to use
the settings you want. Then copy the `.service` and `.timer` files to `~/.config/systemd/user/`.

Enable the timer:

```
systemctl --user start mullvadvpn-app-tests.timer && \
systemctl --user enable mullvadvpn-app-tests.timer
```
