package net.mullvad.mullvadvpn.service

import android.content.Context
import android.os.DeadObjectException
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.os.Messenger
import kotlin.properties.Delegates.observable
import net.mullvad.talpid.ConnectivityListener

class ServiceHandler(
    context: Context,
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

    val appVersionInfoCache = AppVersionInfoCache(context).apply {
        currentVersionNotifier.subscribe(this@ServiceHandler) { currentVersion ->
            sendEvent(Event.CurrentVersion(currentVersion))
        }

        appVersionInfoNotifier.subscribe(this@ServiceHandler) { appVersionInfo ->
            sendEvent(Event.AppVersionInfo(appVersionInfo))
        }
    }

    val authTokenCache = AuthTokenCache().apply {
        authTokenNotifier.subscribe(this@ServiceHandler) { authToken ->
            sendEvent(Event.AuthToken(authToken))
        }
    }

    val customDns = CustomDns(settingsListener)

    val keyStatusListener = KeyStatusListener().apply {
        onKeyStatusChange.subscribe(this@ServiceHandler) { keyStatus ->
            sendEvent(Event.WireGuardKeyStatus(keyStatus))
        }
    }

    val locationInfoCache =
        LocationInfoCache(connectionProxy, connectivityListener, settingsListener).apply {
            onNewLocation = { location ->
                sendEvent(Event.NewLocation(location))
            }
        }

    val relayListListener = RelayListListener().apply {
        relayListNotifier.subscribe(this@ServiceHandler) { relayList ->
            sendEvent(Event.NewRelayList(relayList))
        }
    }

    var daemon by observable<MullvadDaemon?>(null) { _, _, newDaemon ->
        settingsListener.daemon = newDaemon
        accountCache.daemon = newDaemon
        appVersionInfoCache.daemon = newDaemon
        connectionProxy.daemon = newDaemon
        customDns.daemon = newDaemon
        keyStatusListener.daemon = newDaemon
        locationInfoCache.daemon = newDaemon
        relayListListener.daemon = newDaemon
    }

    init {
        connectionProxy.onStateChange.subscribe(this) { tunnelState ->
            sendEvent(Event.TunnelStateChange(tunnelState))
        }

        splitTunneling.onChange.subscribe(this) { excludedApps ->
            sendEvent(Event.SplitTunnelingUpdate(excludedApps))
        }
    }

    override fun handleMessage(message: Message) {
        val request = Request.fromMessage(message)

        when (request) {
            is Request.AddCustomDnsServer -> customDns.addDnsServer(request.address)
            is Request.Connect -> connectionProxy.connect()
            is Request.CreateAccount -> accountCache.createNewAccount()
            is Request.Disconnect -> connectionProxy.disconnect()
            is Request.ExcludeApp -> {
                request.packageName?.let { packageName ->
                    splitTunneling.excludeApp(packageName)
                }
            }
            is Request.FetchAccountExpiry -> accountCache.fetchAccountExpiry()
            is Request.FetchAuthToken -> authTokenCache.fetchNewToken()
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
            is Request.Reconnect -> connectionProxy.reconnect()
            is Request.RegisterListener -> registerListener(request.listener)
            is Request.RemoveAccountFromHistory -> {
                request.account?.let { account ->
                    accountCache.removeAccountFromHistory(account)
                }
            }
            is Request.RemoveCustomDnsServer -> customDns.removeDnsServer(request.address)
            is Request.ReplaceCustomDnsServer -> {
                customDns.replaceDnsServer(request.oldAddress, request.newAddress)
            }
            is Request.SetAccount -> accountCache.account = request.account
            is Request.SetEnableCustomDns -> customDns.setEnabled(request.enable)
            is Request.SetEnableSplitTunneling -> splitTunneling.enabled = request.enable
            is Request.SetRelayLocation -> {
                relayListListener.selectedRelayLocation = request.relayLocation
            }
            is Request.SetWireGuardMtu -> settingsListener.wireguardMtu = request.mtu
            is Request.VpnPermissionResponse -> {
                connectionProxy.vpnPermission.spawnUpdate(request.vpnPermission)
            }
            is Request.WireGuardGenerateKey -> keyStatusListener.generateKey()
            is Request.WireGuardVerifyKey -> keyStatusListener.verifyKey()
        }
    }

    fun onDestroy() {
        accountCache.onDestroy()
        appVersionInfoCache.onDestroy()
        authTokenCache.onDestroy()
        customDns.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        relayListListener.onDestroy()
        settingsListener.onDestroy()

        connectionProxy.onStateChange.unsubscribe(this)
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
            send(Event.CurrentVersion(appVersionInfoCache.currentVersion).message)
            send(Event.AppVersionInfo(appVersionInfoCache.appVersionInfo).message)
            send(Event.NewRelayList(relayListListener.relayList).message)
            send(Event.AuthToken(authTokenCache.authToken).message)
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
