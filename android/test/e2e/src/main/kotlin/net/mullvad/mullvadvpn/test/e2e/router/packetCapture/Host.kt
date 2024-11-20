package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

data class Host(val ipAddress: String, val port: Int) {
    companion object {
        fun fromString(connectionInfo: String): Host {
            val connectionInfoParts = connectionInfo.split(":")
            val ipAddress = connectionInfoParts.first()
            val port = connectionInfoParts.last().toInt()
            return Host(ipAddress, port)
        }
    }
}
