package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import kotlin.time.Duration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking

class TrafficGenerator(val destinationHost: String, val destinationPort: Int) {
    private var sendTrafficJob: Job? = null

    suspend fun generateTraffic(interval: Duration, block: suspend () -> Unit) = runBlocking {
        startGeneratingUDPTraffic(interval)
        block()
        stopGeneratingUDPTraffic()
        return@runBlocking Unit
    }

    private fun startGeneratingUDPTraffic(interval: Duration) {
        val socket = DatagramSocket()
        val address = InetAddress.getByName(destinationHost)
        val data = ByteArray(1024)
        val packet = DatagramPacket(data, data.size, address, destinationPort)

        sendTrafficJob =
            CoroutineScope(Dispatchers.IO).launch {
                while (true) {
                    socket.send(packet)
                    Logger.v(
                        "TrafficGenerator sending UDP packet to $destinationHost:$destinationPort"
                    )
                    delay(interval)
                }
            }
    }

    private fun stopGeneratingUDPTraffic() {
        sendTrafficJob!!.cancel()
    }
}
