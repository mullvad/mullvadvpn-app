You can connect to a guest VM remotely by forwarding a VNC server port over SSH. QEMU comes with a
built-in VNC server. This example starts a Debian 11 VM as the `test` user:

```
ssh -L 5933:127.0.0.1:5933 -tt $SSH_HOST "sudo -u test bash -c 'cd $TEST_APP_PATH; \
    cargo run --bin test-manager run-vm debian11 --vnc 5933'"
```

Replace `$SSH_HOST` with the server that you wish to connect to, and `$TEST_APP_PATH` with the path
to the copy of this repository on the server.

**NOTE**: In the above example, any changes made to the image will be lost. To make permanent
changes, remove the `-snapshot` option.

Afterwards, use a VNC client such as the TigerVNC client to connect to the given port on localhost.
In this example: `127.0.0.1:5933`