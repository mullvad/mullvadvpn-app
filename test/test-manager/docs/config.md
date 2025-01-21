# Introduction

This document outlines the format of the configuration used by `test-manager` to perform end-to-end tests in virtualized environments.

## Format

The configuration is a JSON document with three values:

```json
{
    "mullvad_host": <optional string>,
    "vms": <document>,
    "test_locations": [ {"test_name": ["relay"] }, .. ],
}
```

The configurable values are prone to change, and for the time being it is probably a good idea to get acquainted with the [Rust struct called "Config"](../src/config.rs) from which the configuration is serialized.
To get started, `test-manager` provides the `test-manager config vm set` command to add and edit VM configurations.
It is also recommended to view the [example section](#Examples) further down this document.

## Location

The configuration is assumed to exist in `$XDG_CONFIG_HOME/mullvad-test/config.json` (most likely `$HOME/.config/mullvad-test/config.json`) on Linux and `$HOME/Library/Application Support/mullvad-test/config.json` on macOS.

## Per-test relay selection

It is possible to configure which relay(s) should be selected on a test-per-test basis by providing the "test_locations"
configuration option. If no explicit configuration is given, no assumption will be made from within the tests themselves.

The format is a list of maps with a single key-value pair, where the key is a [glob pattern](<https://en.wikipedia.org/wiki/Glob_(programming)>)
that will be matched against the test name, and the value is a list of locations to use for the matching tests.
The name of the locations are the same as for the `mullvad-cli`.

### Example

```json
{
  // other fields
  "test_locations": [
    { "*daita*": ["se-got-wg-001", "se-got-wg-002"] },
    { "*": ["se"] }
  ]
}
```

The above example will set the locations for the test `test_daita` to a custom list
containing `se-got-wg-001` and `se-got-wg-002`. The `*` is a wildcard that will match
any test name. The configuration is read from top-to-bottom, and the first match will be used.

## Example configurations

### Minimal

The minimal valid configuration does not contain any virtual machines

```json
{
  "mullvad_host": "stagemole.eu",
  "vms": {}
}
```

### Complete

A configuration containing one Debian 12 VM and one Windows 11 VM

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
            "architecture": "x64",
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
}
```
