package net.mullvad.mullvadvpn.model

sealed class TransportProtocol {
    class Tcp : TransportProtocol()
    class Udp : TransportProtocol()
}
