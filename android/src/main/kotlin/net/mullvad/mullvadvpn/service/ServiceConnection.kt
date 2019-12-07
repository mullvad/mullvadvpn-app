package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.talpid.ConnectivityListener

data class ServiceConnection(
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener
)
