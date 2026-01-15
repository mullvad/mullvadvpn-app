package net.mullvad.mullvadvpn.test.mockapi.server

import io.ktor.server.engine.EmbeddedServer
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.netty.NettyApplicationEngine
import io.ktor.util.network.port
import java.net.InetAddress
import kotlinx.coroutines.runBlocking

object MockServer {
    fun createWithRouter(mockApiRouter: MockApiRouter) =
        embeddedServer(factory = Netty, port = 0, host = InetAddress.getLocalHost().hostName) {
            mockApiRouter.setup(this)
        }
}

fun EmbeddedServer<NettyApplicationEngine, *>.port() = runBlocking {
    this@port.engine.resolvedConnectors().first().port
}
