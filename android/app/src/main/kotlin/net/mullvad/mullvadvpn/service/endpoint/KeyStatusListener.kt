package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.KeygenEvent

class KeyStatusListener(endpoint: ServiceEndpoint) {
    companion object {
        private enum class Command {
            GenerateKey,
            VerifyKey,
        }
    }

    private val daemon = endpoint.intermittentDaemon

    private val commandChannel = spawnActor()

    var keyStatus by observable<KeygenEvent?>(null) { _, _, status ->
        endpoint.sendEvent(Event.WireGuardKeyStatus(status))
    }
        private set

    init {
        daemon.registerListener(this) { newDaemon ->
            newDaemon?.apply {
                keyStatus = getWireguardKey()?.let { wireguardKey ->
                    KeygenEvent.NewKey(wireguardKey, null, null)
                }

                onKeygenEvent = { event -> keyStatus = event }
            }
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.WireGuardGenerateKey::class) { _ ->
                commandChannel.sendBlocking(Command.GenerateKey)
            }

            registerHandler(Request.WireGuardVerifyKey::class) { _ ->
                commandChannel.sendBlocking(Command.VerifyKey)
            }
        }
    }

    fun onDestroy() {
        commandChannel.close()
        daemon.unregisterListener(this)
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            for (command in channel) {
                when (command) {
                    Command.GenerateKey -> {
                        // TODO: Skip until device integration is ready.
                        // generateKey()
                    }
                    Command.VerifyKey -> {
                        // TODO: Skip until device integration is ready.
                        // verifyKey()
                    }
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
        }
    }

    private suspend fun generateKey() {
        val oldStatus = keyStatus
        val newStatus = daemon.await().generateWireguardKey()
        val newFailure = newStatus?.failure()
        if (oldStatus is KeygenEvent.NewKey && newFailure != null) {
            keyStatus = KeygenEvent.NewKey(
                oldStatus.publicKey,
                oldStatus.verified,
                newFailure
            )
        } else {
            keyStatus = newStatus ?: KeygenEvent.GenerationFailure
        }
    }

    private suspend fun verifyKey() {
        // Only update verification status if the key is actually there
        (keyStatus as? KeygenEvent.NewKey)?.let { currentStatus ->
            keyStatus = KeygenEvent.NewKey(
                currentStatus.publicKey,
                daemon.await().verifyWireguardKey(),
                currentStatus.replacementFailure
            )
        }
    }
}
