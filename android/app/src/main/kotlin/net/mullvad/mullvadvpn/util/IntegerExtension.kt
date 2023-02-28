package net.mullvad.mullvadvpn.util

fun Int.isValidMtu(): Boolean {
    return this in 1280..1420
}
