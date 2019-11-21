package net.mullvad.talpid.net

sealed class TransportProtocol {
    class Tcp : TransportProtocol()
    class Udp : TransportProtocol()
}
