package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver

import android.net.LocalSocketAddress
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketAddress
import javax.net.SocketFactory

internal class FixedUdsSocketFactory(path: String?, namespace: LocalSocketAddress.Namespace?) :
    SocketFactory() {
    private val localSocketAddress = LocalSocketAddress(path, namespace)

    @Throws(IOException::class)
    override fun createSocket(): Socket {
        return create()
    }

    @Throws(IOException::class)
    override fun createSocket(host: String, port: Int): Socket {
        return createAndConnect()
    }

    @Throws(IOException::class)
    override fun createSocket(
        host: String,
        port: Int,
        localHost: InetAddress,
        localPort: Int
    ): Socket {
        return createAndConnect()
    }

    @Throws(IOException::class)
    override fun createSocket(host: InetAddress, port: Int): Socket {
        return createAndConnect()
    }

    @Throws(IOException::class)
    override fun createSocket(
        address: InetAddress,
        port: Int,
        localAddress: InetAddress,
        localPort: Int
    ): Socket {
        return createAndConnect()
    }

    private fun create(): Socket {
        return FixedUdsSocket(localSocketAddress)
    }

    @Throws(IOException::class)
    private fun createAndConnect(): Socket {
        val socket = create()
        val unusedAddress: SocketAddress = InetSocketAddress(0)
        socket.connect(unusedAddress)
        return socket
    }
}
