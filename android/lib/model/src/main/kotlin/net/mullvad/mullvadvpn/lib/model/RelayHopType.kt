package net.mullvad.mullvadvpn.lib.model

/**
 * Whether this relay is used as an entry or exit hop. A singlehop is considered to be an exit hop,
 * not an entry hop. This is consistent with how recents are displayed (singlehop will show only
 * exit recents, never entry).
 */
enum class RelayHopType {
    ENTRY,
    EXIT,
}
