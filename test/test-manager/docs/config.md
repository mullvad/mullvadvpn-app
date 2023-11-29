# Introduction
This document outlines the format of the configuration used by `test-manager` to perform end-to-end tests in virtualized environments.

# Format
The configuration is a JSON document with two keys:
```json
{
    "mullvad_host": <optional string>,
    "vms": <document>
}
```

The configurable values are prone to change, and for the time being it is probably a good idea to get acquainted with the [Rust struct called "Config"](../src/config.rs) from which the configuration is serialized. To get started, `test-manager` provides the `test-manager set` command to add and edit VM configurations. It is also recommended to view the [example section](#Examples) further down this document.

# Location
The configuration is assumed to exist in `$XDG_CONFIG_HOME/mullvad-test/config.json` (most likely `$HOME/.config/mullvad-test/config.json`) on Linux and `$HOME/Library/Application Support/mullvad-test/config.json` on macOS.

# Examples

## Minimal
The minimal valid configuration does not contain any virtual machines
```json
{
    "mullvad_host": "stagemole.eu",
    "vms": { }
}
```

## Complete
A configuration containing one Debian 12 VM and oen Windows 11 VM
```json
{
    "mullvad_host": "stagemole.eu",
    "vms": {
        "debian12": {
          "vm_type": "qemu",
          "image_path": "$VM_IMAGES/debian12.qcow2",
          "os_type": "linux",
          "package_type": "deb",
          "architecture": "x64",
          "provisioner": "ssh",
          "ssh_user": "test",
          "ssh_password": "test",
          "disks": [],
          "artifacts_dir": "/opt/testing",
          "tpm": false
        },
        "windows11": {
            "vm_type": "qemu",
            "image_path": "$VM_IMAGES/windows11.qcow2",
            "os_type": "windows",
            "package_type": null,
            "architecture": null,
            "provisioner": "noop",
            "ssh_user": null,
            "ssh_password": null,
            "disks": [
              "$TESTRUNNER_IMAGES/windows-test-runner.img"
            ],
            "artifacts_dir": "E:\\",
            "tpm": false
        }
    }
}
```
