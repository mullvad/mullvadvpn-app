package net.mullvad.mullvadvpn.service

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.talpid.util.EventNotifier

class KeyStatusListener(val daemon: MullvadDaemon) {
    val onKeyStatusChange = EventNotifier(getInitialKeyStatus())

    var keyStatus by onKeyStatusChange.notifiable()

    init {
        daemon.onKeygenEvent = { event -> keyStatus = event }
    }

    private fun getInitialKeyStatus(): KeygenEvent? {
        return daemon.getWireguardKey()?.let { wireguardKey ->
            KeygenEvent.NewKey(wireguardKey, null, null)
        }
    }

    fun generateKey() = GlobalScope.launch(Dispatchers.Default) {
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
            keyStatus = newStatus ?: KeygenEvent.GenerationFailure
        }
    }

    fun verifyKey() = GlobalScope.launch(Dispatchers.Default) {
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

    fun onDestroy() {
        daemon.onKeygenEvent = null
        onKeyStatusChange.unsubscribeAll()
    }
}
