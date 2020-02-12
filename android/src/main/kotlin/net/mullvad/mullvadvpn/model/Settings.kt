package net.mullvad.mullvadvpn.model

data class Settings(
    var accountToken: String?,
    var relaySettings: RelaySettings,
    var allowLan: Boolean,
    var autoConnect: Boolean
)
