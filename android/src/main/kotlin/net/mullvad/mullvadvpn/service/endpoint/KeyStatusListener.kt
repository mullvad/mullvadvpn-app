package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.KeygenEvent

class KeyStatusListener(endpoint: ServiceEndpoint) {
    private val daemon = endpoint.intermittentDaemon

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
                generateKey()
            }

            registerHandler(Request.WireGuardVerifyKey::class) { _ ->
                verifyKey()
            }
        }
    }

    fun onDestroy() {
        daemon.unregisterListener(this)
    }

    private fun generateKey() = GlobalScope.launch(Dispatchers.Default) {
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

    private fun verifyKey() = GlobalScope.launch(Dispatchers.Default) {
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
