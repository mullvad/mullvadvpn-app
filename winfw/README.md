# winfw

Windows firewall for the Mullvad app.

The daemon uses it through `talpid-core`'s `firewall::windows` module.

## Organization of sublayers

In its initialized state, the firewall uses two sublayers:

- The baseline sublayer, weighted the highest, so it sees all traffic first.
  It contains a large number of permit-filters, with a different subset of
  them being activated by different policies, and low-weighted block-all
  filters. If traffic is matched by a permit-filter, it is "lifted" out of the
  sublayer and processing is resumed with the next sublayer.
- The DNS sublayer, weighted slightly lower. All DNS is lifted out of the
  baseline sublayer (see the `PermitDns` rule) and restricted here instead:
  only explicitly permitted DNS traffic passes, and a low-weighted filter
  blocks the rest.

When the daemon shuts down while the firewall is in the blocked state, the
ephemeral filters installed here are swapped for persistent ones (see
`talpid-core`'s `firewall::windows::objects::persistent`), which survive a
Base Filtering Engine restart and a reboot.

## Snapshot tests

`tests/snapshot.rs` drives the firewall through every policy state and
compares the resulting WFP object state with the snapshots in
`tests/snapshots/`, guarding the firewall against behavior drift.

The tests talk to the real Base Filtering Engine, so they must run as
administrator, on Windows. They replace every WFP object the firewall owns,
including those of a running Mullvad daemon, so they are ignored by default:

```sh
cargo test -p winfw -- --ignored
```

The snapshots are stored with [insta]. When one legitimately changes,
regenerate them with:

```sh
INSTA_UPDATE=always cargo test -p winfw -- --ignored
```

or use the review workflow, which walks you through the differences one by
one:

```sh
cargo insta test -p winfw -- --ignored
cargo insta review
```

Snapshots are normalized: object GUIDs are shown by name, interface LUIDs are
resolved back to their alias, the tunnel interface is shown as a placeholder,
application paths have their volume number replaced, the engine-set
`FWPM_FILTER_FLAG_INDEXED` bit is masked away, and filters and conditions are
sorted, since WFP does not assign meaning to their order.

[insta]: https://insta.rs
