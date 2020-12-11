package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.talpid.util.EventNotifier

class KeyStatusListener {
    val onKeyStatusChange = EventNotifier<KeygenEvent?>(null)

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, newDaemon ->
        oldDaemon?.onKeygenEvent = null

        newDaemon?.apply {
            keyStatus = getWireguardKey()?.let { wireguardKey ->
                KeygenEvent.NewKey(wireguardKey, null, null)
            }

            onKeygenEvent = { event -> keyStatus = event }
        }
    }

    var keyStatus by onKeyStatusChange.notifiable()

    fun generateKey() = GlobalScope.launch(Dispatchers.Default) {
        daemon?.let { daemon ->
            val oldStatus = keyStatus
            val newStatus = daemon.generateWireguardKey()
            val newFailure = newStatus?.failure()
            if (oldStatus is KeygenEvent.NewKey && newFailure != null) {
                keyStatus = KeygenEvent.NewKey(
                    oldStatus.publicKey,
                    oldStatus.verified,
                    newFailure
                )
            } else {
                keyStatus = newStatus ?: KeygenEvent.GenerationFailure()
            }
        }
    }

    fun verifyKey() = GlobalScope.launch(Dispatchers.Default) {
        daemon?.let { daemon ->
            val verified = daemon.verifyWireguardKey()
            // Only update verification status if the key is actually there
            when (val state = keyStatus) {
                is KeygenEvent.NewKey -> {
                    keyStatus = KeygenEvent.NewKey(
                        state.publicKey,
                        verified,
                        state.replacementFailure
                    )
                }
            }
        }
    }

    fun onDestroy() {
        daemon = null
        onKeyStatusChange.unsubscribeAll()
    }
}
