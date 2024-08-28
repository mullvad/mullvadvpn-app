package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.util.Timer
import java.util.TimerTask

class TrafficGenerator(
    val destinationHost: String,
    val destinationPort: Int
) {
    private var timer: Timer? = null
    private var timerTask: TimerTask? = null

    fun startGeneratingUDPTraffic(interval: Long) {
        val socket = DatagramSocket()
        val address = InetAddress.getByName(destinationHost)
        val data = ByteArray(1024)
        val packet = DatagramPacket(data, data.size, address, destinationPort)

        timer = Timer()
        timerTask =
            object : TimerTask() {
                override fun run() {
                    socket.send(packet)
                    Logger.v(
                        "Traffic generator sending UDP packet to $destinationHost:$destinationPort"
                    )
                }
            }

        timer?.schedule(timerTask, 0, interval)
    }

    fun stopGeneratingUDPTraffic() {
        timerTask?.cancel()
        timer?.cancel()
        timerTask = null
        timer = null
    }
}
