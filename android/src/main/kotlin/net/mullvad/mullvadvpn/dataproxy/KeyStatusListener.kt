package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MullvadDaemon
import net.mullvad.mullvadvpn.model.KeygenEvent

class KeyStatusListener(val asyncDaemon: Deferred<MullvadDaemon>) {
    private var daemon: MullvadDaemon? = null

    private val setUpJob = setUp()

    var keyStatus: KeygenEvent? = null
        private set(value) {
            synchronized(this) {
                field = value

                if (value != null) {
                    onKeyStatusChange?.invoke(value)
                }
            }
        }

    var onKeyStatusChange: ((KeygenEvent) -> Unit)? = null
        set(value) {
            field = value

            synchronized(this) {
                keyStatus?.let { status -> value?.invoke(status) }
            }
        }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        daemon = asyncDaemon.await()
        daemon?.onKeygenEvent = { event -> keyStatus = event }
        val wireguardKey = daemon?.getWireguardKey()
        if (wireguardKey != null) {
            keyStatus = KeygenEvent.NewKey(wireguardKey, null, null)
        }
    }

    fun generateKey() = GlobalScope.launch(Dispatchers.Default) {
            setUpJob.join()
            val oldStatus = keyStatus
            val newStatus = daemon?.generateWireguardKey()
            if (oldStatus is KeygenEvent.NewKey && newStatus is KeygenEvent.Failure) {
                keyStatus = KeygenEvent.NewKey(oldStatus.publicKey,
                                oldStatus.verified,
                                newStatus.failure)
            } else {
                keyStatus = newStatus
            }
    }

    fun verifyKey() = GlobalScope.launch(Dispatchers.Default) {
            setUpJob.join()
            val verified = daemon?.verifyWireguardKey()
            // Only update verification status if the key is actually there
            when (val state = keyStatus) {
                is KeygenEvent.NewKey -> {
                    keyStatus = KeygenEvent.NewKey(state.publicKey,
                                    verified,
                                    state.replacementFailure)
                }
            }
    }

    fun onDestroy() {
        setUpJob.cancel()
        daemon?.onKeygenEvent = null
    }

    private fun retryKeyGeneration() = GlobalScope.launch(Dispatchers.Default) {
        setUpJob.join()
        keyStatus = daemon?.generateWireguardKey()
    }
}
