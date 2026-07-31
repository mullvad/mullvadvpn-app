package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import android.net.LocalSocket
import android.net.LocalSocketAddress
import java.io.File
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.Socket
import java.net.SocketAddress
import javax.net.SocketFactory

/** A [SocketFactory] that creates Unix domain sockets backed by [LocalSocket]. */
class UnixDomainSocketFactory(private val path: String) : SocketFactory() {

    constructor(file: File) : this(file.absolutePath)

    override fun createSocket(): Socket = LocalSocketWrapper(path)

    override fun createSocket(host: String?, port: Int): Socket = createSocket()

    override fun createSocket(
        host: String,
        port: Int,
        localHost: InetAddress,
        localPort: Int,
    ): Socket = createSocket()

    override fun createSocket(host: InetAddress?, port: Int): Socket = createSocket()

    override fun createSocket(
        host: InetAddress?,
        port: Int,
        localAddress: InetAddress?,
        localPort: Int,
    ): Socket = createSocket()
}

/** Wraps [LocalSocket] as a [Socket] so OkHttp can use it. */
private class LocalSocketWrapper(path: String) : Socket() {
    private val localSocket = LocalSocket()

    init {
        if (!localSocket.isConnected) {
            localSocket.connect(LocalSocketAddress(path, LocalSocketAddress.Namespace.FILESYSTEM))
        }
    }

    override fun connect(endpoint: SocketAddress?, timeout: Int) {
        // No-op, already connected in init
    }

    override fun getInputStream(): InputStream = localSocket.inputStream

    override fun getOutputStream(): OutputStream = localSocket.outputStream

    override fun close() = localSocket.close()

    override fun isClosed(): Boolean = !localSocket.isConnected

    override fun isConnected(): Boolean = localSocket.isConnected

    override fun setSoTimeout(timeout: Int) {
        localSocket.soTimeout = timeout
    }

    override fun getSoTimeout(): Int = localSocket.soTimeout

    override fun shutdownInput() = localSocket.shutdownInput()

    override fun shutdownOutput() = localSocket.shutdownOutput()

    override fun isInputShutdown(): Boolean =
        false // Not supported by LocalSocket, always return false

    override fun isOutputShutdown(): Boolean =
        false // Not supported by LocalSocket, always return false
}
