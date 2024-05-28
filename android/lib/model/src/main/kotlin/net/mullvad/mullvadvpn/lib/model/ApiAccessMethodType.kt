package net.mullvad.mullvadvpn.lib.model

sealed interface ApiAccessMethodType {
    data object Direct : ApiAccessMethodType

    data object Bridges : ApiAccessMethodType

    sealed interface CustomProxy : ApiAccessMethodType {
        data class Socks5Local(
            val remoteIp: String,
            val remotePort: Port,
            val remoteTransportProtocol: TransportProtocol,
            val localPort: Port
        ) : CustomProxy

        data class Socks5Remote(val ip: String, val port: Port, val auth: SocksAuth) : CustomProxy

        data class Shadowsocks(
            val ip: String,
            val port: Port,
            val password: String,
            val cipher: String
        ) : CustomProxy
    }
}
