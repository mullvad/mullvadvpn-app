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
```bash
$ deplay.sh ./mullvad_2025.13.x86_64.ipk
```

## Note

`2025.13` is used as an example version.
