package net.mullvad.mullvadvpn.lib.daemon.grpc

import io.grpc.EquivalentAddressGroup
import io.grpc.NameResolver
import java.net.InetAddress
import java.net.InetSocketAddress
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class SimpleDnsNameResolver(
    val authority: String?,
    val name: String,
    val args: Args,
    private val scope: CoroutineScope
) : NameResolver() {

    override fun getServiceAuthority(): String? {
        return "localhost"
    }

    override fun start(listener: Listener2) {
        scope.launch(context = Dispatchers.IO) {
            val builder = ResolutionResult.newBuilder()
            builder.setAddresses(
                listOf(EquivalentAddressGroup(InetSocketAddress(InetAddress.getLocalHost(), 80)))
            )
            listener.onResult(builder.build())
        }
    }

    override fun shutdown() {
        // scope.cancel("Shoutdown")
    }
}
