package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver

import io.grpc.EquivalentAddressGroup
import io.grpc.NameResolver
import java.net.InetSocketAddress

class DummyNameResolver : NameResolver() {

    override fun getServiceAuthority(): String = SERVICE_AUTHORITY

    override fun start(listener: Listener2) {
        val resolutionResult =
            ResolutionResult.newBuilder()
                .setAddresses(
                    listOf(
                        EquivalentAddressGroup(
                            InetSocketAddress.createUnresolved(DUMMY_HOST, DUMMY_PORT)
                        )
                    )
                )
                .build()

        listener.onResult(resolutionResult)
    }

    override fun shutdown() {
        // Do nothing
    }

    companion object {
        const val SERVICE_AUTHORITY = "localhost"
        private const val DUMMY_HOST = ""
        private const val DUMMY_PORT = 80
    }
}
