package net.mullvad.mullvadvpn.service

import android.os.DeadObjectException
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.talpid.ConnectivityListener

class ServiceHandler(looper: Looper, connectivityListener: ConnectivityListener) : Handler(looper) {
    private val listeners = mutableListOf<Messenger>()

    val settingsListener = SettingsListener().apply {
        subscribe(this@ServiceHandler) { settings ->
            sendEvent(Event.SettingsUpdate(settings))
        }
    }

    val accountCache = AccountCache(settingsListener)

    val keyStatusListener = KeyStatusListener().apply {
        onKeyStatusChange.subscribe(this@ServiceHandler) { keyStatus ->
            sendEvent(Event.WireGuardKeyStatus(keyStatus))
        }
    }

    val locationInfoCache = LocationInfoCache(connectivityListener, settingsListener).apply {
        onNewLocation = { location ->
            sendEvent(Event.NewLocation(location))
        }
    }

    var daemon by observable<MullvadDaemon?>(null) { _, _, newDaemon ->
        settingsListener.daemon = newDaemon
        accountCache.daemon = newDaemon
        keyStatusListener.daemon = newDaemon
        locationInfoCache.daemon = newDaemon
    }

    override fun handleMessage(message: Message) {
        val request = Request.fromMessage(message)

        when (request) {
            is Request.CreateAccount -> accountCache.createNewAccount()
            is Request.FetchAccountExpiry -> accountCache.fetchAccountExpiry()
            is Request.InvalidateAccountExpiry -> {
                accountCache.invalidateAccountExpiry(request.expiry)
            }
            is Request.Login -> request.account?.let { account -> accountCache.login(account) }
            is Request.Logout -> accountCache.logout()
            is Request.RegisterListener -> registerListener(request.listener)
            is Request.RemoveAccountFromHistory -> {
                request.account?.let { account ->
                    accountCache.removeAccountFromHistory(account)
                }
            }
            is Request.WireGuardGenerateKey -> keyStatusListener.generateKey()
            is Request.WireGuardVerifyKey -> keyStatusListener.verifyKey()
        }
    }

    private fun registerListener(listener: Messenger) {
        listeners.add(listener)

        listener.apply {
            send(Event.SettingsUpdate(settingsListener.settings).message)
            send(Event.NewLocation(locationInfoCache.location).message)
            send(Event.WireGuardKeyStatus(keyStatusListener.keyStatus).message)
        }
    }

    private fun sendEvent(event: Event) {
        val deadListeners = mutableListOf<Messenger>()

        for (listener in listeners) {
            try {
                listener.send(event.message)
            } catch (_: DeadObjectException) {
                deadListeners.add(listener)
            }
        }

        for (deadListener in deadListeners) {
            listeners.remove(deadListener)
        }
    }
}
