# Bootstrapping an OpenWRT VM
The `bootstrap.sh` script will download an OpenWRT 24.10 x86_64 image and decompress it to a `.qcow2` file, suitable for running with QEMU.
The `.qcow2` file will be dumped in the same folder from which `bootstrap.sh` is run, and it will be named `openwrt.qcow2`.
The following paragraphs assume that the file is located in this folder.

The VM will be allocated a disk of 1GB by default. To change this, modify `bootstrap.sh`.

# Running the VM
Running `debug.sh` will start an ephemeral VM.
The VM will run in the terminal and forward ssh over port `1337` on localhost.
There is only a root user with no password.

# Configuring the VM
This should only have to be done once! Start the vm in persistant mode, meaning that changes are permanent.

```bash
$ bash persistant.sh
```

## Network
Edit `/etc/config/network` and change the following options for `config interface 'lan'`:

- change option proto 'static' to 'dhcp'
- remove IP address and netmask setting

and run `/etc/init.d/network restart`.

See https://sven.stormbind.net/blog/posts/deb_qemu_local_openwrt/ for more information.

## Packages
To be able to `scp` files to the VM, install an `stfp` server.

```bash
$ opkg update
$ opkg install openssh-sftp-server
```

## Save
To save these changes, turn off the VM.

```bash
$ poweroff
```
