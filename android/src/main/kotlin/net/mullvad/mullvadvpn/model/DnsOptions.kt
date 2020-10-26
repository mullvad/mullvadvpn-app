package net.mullvad.mullvadvpn.model

import java.net.InetAddress
import java.util.ArrayList

data class DnsOptions(val custom: Boolean, val addresses: ArrayList<InetAddress>)
