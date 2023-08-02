package net.mullvad.talpid.util

import java.net.InetAddress

fun InetAddress.addressString(): String {
    val hostNameAndAddress = this.toString().split('/', limit = 2)
    val address = hostNameAndAddress[1]

    return address
}
