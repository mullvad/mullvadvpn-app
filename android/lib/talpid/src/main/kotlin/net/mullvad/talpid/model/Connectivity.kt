package net.mullvad.talpid.model

sealed class Connectivity {
    data class Status(val ipv4: Boolean, val ipv6: Boolean) : Connectivity()
}
