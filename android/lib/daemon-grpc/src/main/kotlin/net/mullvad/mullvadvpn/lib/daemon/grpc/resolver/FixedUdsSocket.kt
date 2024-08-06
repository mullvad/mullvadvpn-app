package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver

import android.net.LocalSocket
import android.net.LocalSocketAddress
import com.google.errorprone.annotations.concurrent.GuardedBy
import java.io.FilterInputStream
import java.io.FilterOutputStream
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.Socket
import java.net.SocketAddress
import java.net.SocketException
import java.nio.channels.SocketChannel

internal class FixedUdsSocket (private val localSocketAddress: LocalSocketAddress) : Socket() {
 private val localSocket = LocalSocket()

    @GuardedBy("this")
    private var closed = false

    @GuardedBy("this")
    private var inputShutdown = false

    @GuardedBy("this")
    private var outputShutdown = false

    override fun bind(bindpoint: SocketAddress) {
        // no-op
    }

    @Synchronized @Throws(IOException::class)  override fun close() {
        if (closed) {
            return
        }
        if (!inputShutdown) {
            shutdownInput()
        }
        if (!outputShutdown) {
            shutdownOutput()
        }
        localSocket.close()
        closed = true
    }

    @Throws(IOException::class)  override fun connect(endpoint: SocketAddress) {
        localSocket.connect(localSocketAddress)
    }

    @Throws(IOException::class)  override fun connect(endpoint: SocketAddress, timeout: Int) {
        localSocket.connect(localSocketAddress, timeout)
    }

    override fun getChannel(): SocketChannel {
        throw UnsupportedOperationException("getChannel() not supported")
    }

    override fun getInetAddress(): InetAddress {
        throw UnsupportedOperationException("getInetAddress() not supported")
    }

    @Throws(IOException::class)  override fun getInputStream(): InputStream {
        return object : FilterInputStream(localSocket.inputStream) {
            @Throws(IOException::class)  override fun close() {
                this@FixedUdsSocket .close()
            }
        }
    }

    override fun getKeepAlive(): Boolean {
        throw UnsupportedOperationException("Unsupported operation getKeepAlive()")
    }

    override fun getLocalAddress(): InetAddress {
        throw UnsupportedOperationException("Unsupported operation getLocalAddress()")
    }

    override fun getLocalPort(): Int {
        throw UnsupportedOperationException("Unsupported operation getLocalPort()")
    }

    override fun getLocalSocketAddress(): SocketAddress {
        return object : SocketAddress() {}
    }

    override fun getOOBInline(): Boolean {
        throw UnsupportedOperationException("Unsupported operation getOOBInline()")
    }

    @Throws(IOException::class)  override fun getOutputStream(): OutputStream {
        return object : FilterOutputStream(localSocket.outputStream) {
            @Throws(IOException::class)  override fun close() {
                this@FixedUdsSocket .close()
            }
        }
    }

    override fun getPort(): Int {
        throw UnsupportedOperationException("Unsupported operation getPort()")
    }

    @Throws(SocketException::class)  override fun getReceiveBufferSize(): Int {
        try  {
            return localSocket.receiveBufferSize
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun getRemoteSocketAddress(): SocketAddress {
        return object : SocketAddress() {}
    }

    override fun getReuseAddress(): Boolean {
        throw UnsupportedOperationException("Unsupported operation getReuseAddress()")
    }

    @Throws(SocketException::class)  override fun getSendBufferSize(): Int {
        try  {
            return localSocket.sendBufferSize
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun getSoLinger(): Int {
        return -1 // unsupported
    }

    @Throws(SocketException::class)  override fun getSoTimeout(): Int {
        try  {
            return localSocket.soTimeout
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun getTcpNoDelay(): Boolean {
        return true
    }

    override fun getTrafficClass(): Int {
        throw UnsupportedOperationException("Unsupported operation getTrafficClass()")
    }

    override fun isBound(): Boolean {
        return localSocket.isBound
    }

    @Synchronized  override fun isClosed(): Boolean {
        return closed
    }

    override fun isConnected(): Boolean {
        return localSocket.isConnected
    }

    @Synchronized  override fun isInputShutdown(): Boolean {
        return inputShutdown
    }

    @Synchronized  override fun isOutputShutdown(): Boolean {
        return outputShutdown
    }

    override fun sendUrgentData(data: Int) {
        throw UnsupportedOperationException("Unsupported operation sendUrgentData()")
    }

    override fun setKeepAlive(on: Boolean) {
        throw UnsupportedOperationException("Unsupported operation setKeepAlive()")
    }

    override fun setOOBInline(on: Boolean) {
        throw UnsupportedOperationException("Unsupported operation setOOBInline()")
    }

    override fun setPerformancePreferences(connectionTime: Int, latency: Int, bandwidth: Int) {
        throw UnsupportedOperationException("Unsupported operation setPerformancePreferences()")
    }

    @Throws(SocketException::class)  override fun setReceiveBufferSize(size: Int) {
        try  {
            localSocket.receiveBufferSize = size
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun setReuseAddress(on: Boolean) {
        throw UnsupportedOperationException("Unsupported operation setReuseAddress()")
    }

    @Throws(SocketException::class)  override fun setSendBufferSize(size: Int) {
        try  {
            localSocket.sendBufferSize = size
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun setSoLinger(on: Boolean, linger: Int) {
        throw UnsupportedOperationException("Unsupported operation setSoLinger()")
    }

    @Throws(SocketException::class)  override fun setSoTimeout(timeout: Int) {
        try  {
            localSocket.soTimeout = timeout
        }catch (e: IOException) {
            throw toSocketException(e)
        }
    }

    override fun setTcpNoDelay(on: Boolean) {
        // no-op
    }

    override fun setTrafficClass(tc: Int) {
        throw UnsupportedOperationException("Unsupported operation setTrafficClass()")
    }

    @Synchronized @Throws(IOException::class)  override fun shutdownInput() {
        localSocket.shutdownInput()
        inputShutdown = true
    }

    @Synchronized @Throws(IOException::class)  override fun shutdownOutput() {
        localSocket.shutdownOutput()
        outputShutdown = true
    }

    companion object {
        private fun toSocketException(e: Throwable): SocketException {
             val se = SocketException()
            se.initCause(e)
            return se
        }
    }}

