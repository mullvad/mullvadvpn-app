package net.mullvad.mullvadvpn.lib.grpc.util

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
private class LocalSocketWrapper(private val path: String) : Socket() {
    private val localSocket = LocalSocket()

    // OkHttp calls setSoTimeout() before connect(), but LocalSocket doesn't create its
    // underlying file descriptor until connect() succeeds, so setSoTimeout() would throw
    // "socket not created" if applied too early. Buffer the value and apply it once connected.
    private var pendingSoTimeout: Int? = null

    override fun connect(endpoint: SocketAddress?, timeout: Int) {
        if (!localSocket.isConnected) {
            localSocket.connect(LocalSocketAddress(path, LocalSocketAddress.Namespace.FILESYSTEM))
            pendingSoTimeout?.let { localSocket.soTimeout = it }
        }
    }

    override fun getInputStream(): InputStream = localSocket.inputStream

    override fun getOutputStream(): OutputStream = localSocket.outputStream

    override fun close() = localSocket.close()

    override fun isClosed(): Boolean = !localSocket.isConnected

    override fun isConnected(): Boolean = localSocket.isConnected

    override fun setSoTimeout(timeout: Int) {
        if (localSocket.isConnected) {
            localSocket.soTimeout = timeout
        } else {
            pendingSoTimeout = timeout
        }
    }

    override fun getSoTimeout(): Int =
        if (localSocket.isConnected) localSocket.soTimeout else pendingSoTimeout ?: 0

    override fun shutdownInput() = localSocket.shutdownInput()

    override fun shutdownOutput() = localSocket.shutdownOutput()

    override fun isInputShutdown(): Boolean =
        false // Not supported by LocalSocket, always return false

    override fun isOutputShutdown(): Boolean =
        false // Not supported by LocalSocket, always return false
}
