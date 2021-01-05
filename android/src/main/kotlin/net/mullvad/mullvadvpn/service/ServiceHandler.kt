package net.mullvad.mullvadvpn.service

import android.os.DeadObjectException
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.talpid.ConnectivityListener

class ServiceHandler(
    looper: Looper,
    val connectionProxy: ConnectionProxy,
    connectivityListener: ConnectivityListener,
    val splitTunneling: SplitTunneling
) : Handler(looper) {
    private val listeners = mutableListOf<Messenger>()

    val settingsListener = SettingsListener().apply {
        subscribe(this@ServiceHandler) { settings ->
            sendEvent(Event.SettingsUpdate(settings))
        }
    }

    val accountCache = AccountCache(settingsListener).apply {
        onAccountHistoryChange.subscribe(this@ServiceHandler) { history ->
            sendEvent(Event.AccountHistory(history))
        }

        onLoginStatusChange.subscribe(this@ServiceHandler) { status ->
            sendEvent(Event.LoginStatus(status))
        }
    }

    val keyStatusListener = KeyStatusListener().apply {
        onKeyStatusChange.subscribe(this@ServiceHandler) { keyStatus ->
            sendEvent(Event.WireGuardKeyStatus(keyStatus))
        }
    }

    val locationInfoCache = LocationInfoCache(connectivityListener, settingsListener).apply {
        stateEvents = connectionProxy.onStateChange

        onNewLocation = { location ->
            sendEvent(Event.NewLocation(location))
        }
    }

    var daemon by observable<MullvadDaemon?>(null) { _, _, newDaemon ->
        settingsListener.daemon = newDaemon
        accountCache.daemon = newDaemon
        connectionProxy.daemon = newDaemon
        keyStatusListener.daemon = newDaemon
        locationInfoCache.daemon = newDaemon
    }

    init {
        splitTunneling.onChange.subscribe(this) { excludedApps ->
            sendEvent(Event.SplitTunnelingUpdate(excludedApps))
        }
    }

    override fun handleMessage(message: Message) {
        val request = Request.fromMessage(message)

        when (request) {
            is Request.CreateAccount -> accountCache.createNewAccount()
            is Request.ExcludeApp -> {
                request.packageName?.let { packageName ->
                    splitTunneling.excludeApp(packageName)
                }
            }
            is Request.FetchAccountExpiry -> accountCache.fetchAccountExpiry()
            is Request.IncludeApp -> {
                request.packageName?.let { packageName ->
                    splitTunneling.includeApp(packageName)
                }
            }
            is Request.InvalidateAccountExpiry -> {
                accountCache.invalidateAccountExpiry(request.expiry)
            }
            is Request.Login -> request.account?.let { account -> accountCache.login(account) }
            is Request.Logout -> accountCache.logout()
            is Request.PersistExcludedApps -> splitTunneling.persist()
            is Request.RegisterListener -> registerListener(request.listener)
            is Request.RemoveAccountFromHistory -> {
                request.account?.let { account ->
                    accountCache.removeAccountFromHistory(account)
                }
            }
            is Request.SetEnableSplitTunneling -> splitTunneling.enabled = request.enable
            is Request.WireGuardGenerateKey -> keyStatusListener.generateKey()
            is Request.WireGuardVerifyKey -> keyStatusListener.verifyKey()
        }
    }

    fun onDestroy() {
        accountCache.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        settingsListener.onDestroy()

        splitTunneling.onChange.unsubscribe(this)

        daemon = null
    }

    private fun registerListener(listener: Messenger) {
        listeners.add(listener)

        listener.apply {
            send(Event.LoginStatus(accountCache.onLoginStatusChange.latestEvent).message)
            send(Event.AccountHistory(accountCache.onAccountHistoryChange.latestEvent).message)
            send(Event.SettingsUpdate(settingsListener.settings).message)
            send(Event.NewLocation(locationInfoCache.location).message)
            send(Event.WireGuardKeyStatus(keyStatusListener.keyStatus).message)
            send(Event.SplitTunnelingUpdate(splitTunneling.onChange.latestEvent).message)
            send(Event.ListenerReady().message)
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
