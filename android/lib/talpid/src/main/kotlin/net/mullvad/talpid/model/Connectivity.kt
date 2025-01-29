package net.mullvad.talpid.model

sealed class Connectivity {
    data class Status(val ipv4Available: Boolean, val ipv6Available: Boolean) : Connectivity()

    data object PresumeOnline : Connectivity()
}
