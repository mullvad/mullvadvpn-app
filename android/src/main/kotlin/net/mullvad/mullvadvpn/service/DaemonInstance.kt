package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking

class DaemonInstance(val vpnService: MullvadVpnService, val listener: (MullvadDaemon?) -> Unit) {
    private enum class Command {
        START,
        STOP,
    }

    private val commandChannel = spawnActor()

    private var daemon by observable<MullvadDaemon?>(null) { _, _, newInstance ->
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
}
