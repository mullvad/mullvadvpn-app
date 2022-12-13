package net.mullvad.mullvadvpn.service

import java.io.File
import kotlin.properties.Delegates.observable
import kotlin.reflect.KClass
import kotlin.reflect.safeCast
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.util.Intermittent

private const val RELAYS_FILE = "relays.json"

class DaemonInstance(
    val vpnService: MullvadVpnService
) {
    sealed class Command {
        data class Start(val apiEndpointConfiguration: ApiEndpointConfiguration) : Command()
        object Stop : Command()
    }

    private val commandChannel = spawnActor()

    private var daemon by observable<MullvadDaemon?>(null) { _, oldInstance, _ ->
        oldInstance?.onDestroy()
    }

    val intermittentDaemon = Intermittent<MullvadDaemon>()

    fun start(apiEndpointConfiguration: ApiEndpointConfiguration) {
        commandChannel.trySendBlocking(Command.Start(apiEndpointConfiguration))
    }

    fun stop() {
        commandChannel.trySendBlocking(Command.Stop)
    }

    fun onDestroy() {
        commandChannel.close()
        intermittentDaemon.onDestroy()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        var isRunning = true

        prepareFiles()

        while (isRunning) {
            val startCommand = waitForCommand(channel, Command.Start::class) ?: break
            startDaemon(startCommand.apiEndpointConfiguration)
            isRunning = waitForCommand(channel, Command.Stop::class) is Command.Stop
            stopDaemon()
        }
    }

    private suspend fun <T : Command> waitForCommand(
        channel: ReceiveChannel<Command>,
        command: KClass<T>
    ): T? {
        return try {
            var receivedCommand: T?
            do {
                receivedCommand = command.safeCast(channel.receive())
            } while (receivedCommand == null)
            receivedCommand
        } catch (exception: ClosedReceiveChannelException) {
            null
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
            extract(RELAYS_FILE, shouldOverwriteRelayList)
        }
    }

    private suspend fun startDaemon(
        apiEndpointConfiguration: ApiEndpointConfiguration
    ) {
        val newDaemon = MullvadDaemon(vpnService, apiEndpointConfiguration).apply {
            onDaemonStopped = {
                intermittentDaemon.spawnUpdate(null)
                daemon = null
            }
        }

        daemon = newDaemon
        intermittentDaemon.update(newDaemon)
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
