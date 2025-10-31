# Step 0
Set up an OpenWRT VM (or physical box). `deploy.sh` assumes that it has ssh forwarded on port `1337` on localhost.

## Bootstrapping a VM
See [vm/](vm/README.md) for getting started with a VM quickly!

# Step 1
```bash
$ build.sh --release
```

# Step 2
Yes, this has to be run as `root`.

```bash
$ sudo package.sh 2025.13
```

Ultimately I followed [this guide](https://raymii.org/s/tutorials/Building_IPK_packages_by_hand.html) to learn how to create an `.ipk`.

# Step 3
Start the VM in another terminal.

```bash
$ bash vm/debug.sh
```

# Step 4
Copy over the `.ipk` to the OpenWRT host.

```bash
$ deploy.sh ./mullvad_2025.13.x86_64.ipk
```

# Step 5
ssh into the VM and play around!

```bash
$ bash vm/connect.sh

# In the ssh session / VM / router
# opkg update && opkg install ./mullvad_2025.13.x86_64.ipk
```

## Note

`2025.13` is used as an example version.
