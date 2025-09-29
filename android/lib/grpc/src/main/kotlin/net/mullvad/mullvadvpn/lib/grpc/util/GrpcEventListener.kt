package net.mullvad.mullvadvpn.lib.grpc.util

import java.io.IOException
import java.net.InetSocketAddress
import java.net.Proxy
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.grpc.GrpcConnectivityState
import okhttp3.Call
import okhttp3.Connection
import okhttp3.EventListener
import okhttp3.Protocol
import okhttp3.internal.http2.StreamResetException

internal class GrpcEventListener : EventListener() {
    private val _connectionState =
        MutableStateFlow<GrpcConnectivityState>(GrpcConnectivityState.Closed)
    val connectionState: StateFlow<GrpcConnectivityState> = _connectionState.asStateFlow()

    override fun connectStart(call: Call, inetSocketAddress: InetSocketAddress, proxy: Proxy) {
        _connectionState.update { GrpcConnectivityState.Connecting }
    }

    override fun connectionAcquired(call: Call, connection: Connection) {
        _connectionState.update { GrpcConnectivityState.Ready }
    }

    override fun connectFailed(
        call: Call,
        inetSocketAddress: InetSocketAddress,
        proxy: Proxy,
        protocol: Protocol?,
        ioe: IOException,
    ) {
        _connectionState.update { GrpcConnectivityState.Failed }
    }

    override fun callFailed(call: Call, ioe: IOException) {
        // If we call failed in an expected manner, we can assume the connection is closed.
        if (call.isCanceled() || ioe is StreamResetException) {
            _connectionState.update { GrpcConnectivityState.Closed }
        } else {
            _connectionState.update { GrpcConnectivityState.Failed }
        }
    }
}
