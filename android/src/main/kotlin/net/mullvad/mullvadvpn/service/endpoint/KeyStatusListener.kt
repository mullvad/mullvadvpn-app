package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event.WireGuardKeyStatus
import net.mullvad.mullvadvpn.ipc.Request.WireGuardGenerateKey
import net.mullvad.mullvadvpn.ipc.Request.WireGuardVerifyKey
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.service.endpoint.KeyStatusListener.Companion.Command

class KeyStatusListener(endpoint: ServiceEndpoint) : Actor<Command>() {
    companion object {
        enum class Command {
            GenerateKey,
            VerifyKey,
        }
    }

    private val daemon = endpoint.intermittentDaemon

    var keyStatus by observable<KeygenEvent?>(null) { _, _, status ->
        endpoint.sendEvent(WireGuardKeyStatus(status))
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

        endpoint.dispatcher.run {
            registerHandler(WireGuardGenerateKey::class) { sendBlocking(Command.GenerateKey) }

            registerHandler(WireGuardVerifyKey::class) { sendBlocking(Command.VerifyKey) }
        }
    }

    fun onDestroy() {
        closeActor()
        daemon.unregisterListener(this)
    }

    override suspend fun onNewCommand(command: Command) = when (command) {
        Command.GenerateKey -> generateKey()
        Command.VerifyKey -> verifyKey()
    }

    private suspend fun generateKey() {
        val oldStatus = keyStatus
        val newStatus = daemon.await().generateWireguardKey()
        val newFailure = newStatus?.failure()
        keyStatus = if (oldStatus is KeygenEvent.NewKey && newFailure != null) {
            KeygenEvent.NewKey(oldStatus.publicKey, oldStatus.verified, newFailure)
        } else {
            newStatus ?: KeygenEvent.GenerationFailure
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
