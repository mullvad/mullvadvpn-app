# Settings patches

Mullvad settings patch is a JSON format used to apply changes to Mullvad app settings. The purpose
is to make it easy to share or distribute useful configurations of the app, for example to make it
work better in censored locations.

A patch consists of a JSON object. Each key in the object refers to a setting to be edited by the
patch. The type of the value depends on the setting/key itself. How the setting is updated (merge
strategy) also depends on the setting/key.

If any part of a patch is not supported on a platform (or any platform), for any reason, the app
must reject the entire patch and not apply any changes to the settings. If possible, the reason for
the rejection should be clear to the user.

The format is shared by apps on all platforms. Not all platforms are guaranteed to support all
settings at all times, however.

## Supported settings

Only a subset of all available app settings are patchable, and all of these settings are described
below.

### Relay overrides

The following settings patch sets the *relay IP override* setting for servers `a` and `b`:

```json
{
    "relay_overrides": [
        { "hostname": "a", "ipv4_addr_in": "1.2.3.4" },
        { "hostname": "b", "ipv4_addr_in": "1.2.3.4", "ipv6_addr_in": "::1" }
    ]
}
```

The merge strategy for overrides is "append or replace":

* Overrides for hostnames not present in the array must remain unchanged. For example, in the above
  patch, overrides for some hostname `c` are completely unaffected.
* For any given hostname, only specified overrides should change. For example, the above patch has
  no effect on the value of `ipv6_addr_in` for the hostname `a`.
* Overrides that *are* specified are added or replaced. For example, `ipv4_addr_in` should be set
  to `1.2.3.4`, regardless of what the override was previously set to.

There is no way to remove an existing override (without replacing it) using a patch.

## Versioning and backward compatibility

Patches are not versioned as backward compatibility is not considered important, though
compatibility should not be broken for no good reason.

## Security

Patches must not edit any settings that may compromise security. For example, enabling custom DNS
should not be allowed.

## Examples

See [patch-examples](./patch-examples) for examples of patch files.
