package net.mullvad.mullvadvpn.service

import java.io.File
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking

private const val API_IP_ADDRESS_FILE = "api-ip-address.txt"
private const val RELAYS_FILE = "relays.json"

class DaemonInstance(val vpnService: MullvadVpnService, val listener: (MullvadDaemon?) -> Unit) {
    private enum class Command {
        START,
        STOP,
    }

    private val commandChannel = spawnActor()

    private var daemon by observable<MullvadDaemon?>(null) { _, oldInstance, newInstance ->
        oldInstance?.onDestroy()
        listener(newInstance)
    }

    fun start() {
        commandChannel.sendBlocking(Command.START)
    }

    fun stop() {
        commandChannel.sendBlocking(Command.STOP)
    }

    fun onDestroy() {
        commandChannel.close()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        var isRunning = true

        prepareFiles()
        vpnService.splitTunneling.join()

        while (isRunning) {
            if (!waitForCommand(channel, Command.START)) {
                break
            }

            startDaemon()

            isRunning = waitForCommand(channel, Command.STOP)

            stopDaemon()
        }
    }

    private suspend fun waitForCommand(
        channel: ReceiveChannel<Command>,
        command: Command
    ): Boolean {
        try {
            while (channel.receive() != command) {
                // Wait for command
            }

            return true
        } catch (exception: ClosedReceiveChannelException) {
            return false
        }
    }

    private fun prepareFiles() {
        FileMigrator(File("/data/data/net.mullvad.mullvadvpn"), vpnService.filesDir).apply {
            migrate(RELAYS_FILE)
            migrate("settings.json")
            migrate("daemon.log")
            migrate("daemon.old.log")
            migrate("wireguard.log")
            migrate("wireguard.old.log")
        }

        val shouldOverwriteRelayList =
            lastUpdatedTime() > File(vpnService.filesDir, RELAYS_FILE).lastModified()

        FileResourceExtractor(vpnService).apply {
            extract(API_IP_ADDRESS_FILE, false)
            extract(RELAYS_FILE, shouldOverwriteRelayList)
        }
    }

    private fun startDaemon() {
        daemon = MullvadDaemon(vpnService).apply {
            onDaemonStopped = {
                daemon = null
            }
        }
    }

    private fun stopDaemon() {
        daemon?.shutdown()
    }

    private fun lastUpdatedTime(): Long {
        return vpnService.run {
            packageManager.getPackageInfo(packageName, 0).lastUpdateTime
        }
    }
}
