package net.mullvad.mullvadvpn.lib.model

data class MultihopConstraints(
    val entryConstraints: EntryConstraints = EntryConstraints(),
    val exitConstraints: ExitConstraints = ExitConstraints(),
)
