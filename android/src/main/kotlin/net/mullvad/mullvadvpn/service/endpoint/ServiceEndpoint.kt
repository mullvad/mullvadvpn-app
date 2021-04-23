package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.ipc.RequestDispatcher
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.persistence.SplitTunnelingPersistence
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.ConnectivityListener

class ServiceEndpoint(
    looper: Looper,
    internal val intermittentDaemon: Intermittent<MullvadDaemon>,
    val connectivityListener: ConnectivityListener,
    context: Context
) : Actor<Messenger>() {
    private val listeners = mutableSetOf<Messenger>()

    val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    private val vpnPermission = VpnPermission(context, this)

    val connectionProxy = ConnectionProxy(vpnPermission, this)
    val settingsListener = SettingsListener(this)

    val accountCache = AccountCache(this)
    private val appVersionInfoCache = AppVersionInfoCache(this)
    private val authTokenCache = AuthTokenCache(this)
    private val customDns = CustomDns(this)
    private val keyStatusListener = KeyStatusListener(this)
    private val locationInfoCache = LocationInfoCache(this)
    private val relayListListener = RelayListListener(this)
    val splitTunneling = SplitTunneling(SplitTunnelingPersistence(context), this)
    private val voucherRedeemer = VoucherRedeemer(this)

    init {
        dispatcher.registerHandler(Request.RegisterListener::class) { request ->
            sendBlocking(request.listener)
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
        closeActor()

        accountCache.onDestroy()
        appVersionInfoCache.onDestroy()
        authTokenCache.onDestroy()
        connectionProxy.onDestroy()
        customDns.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        relayListListener.onDestroy()
        settingsListener.onDestroy()
        splitTunneling.onDestroy()
        voucherRedeemer.onDestroy()
    }

    internal fun sendEvent(event: Event) {
        synchronized(this) {
            val deadListeners = mutableSetOf<Messenger>()

            for (listener in listeners) {
                try {
                    listener.send(event.message)
                } catch (_: DeadObjectException) {
                    deadListeners.add(listener)
                }
            }

            deadListeners.forEach { listeners.remove(it) }
        }
    }

    override suspend fun onNewCommand(command: Messenger) {
        intermittentDaemon.await()
        registerListener(command)
    }

    private fun registerListener(listener: Messenger) {
        synchronized(this) {
            listeners.add(listener)

            sequenceOf(
                Event.TunnelStateChange(connectionProxy.state),
                Event.LoginStatus(accountCache.onLoginStatusChange.latestEvent),
                Event.AccountHistory(accountCache.onAccountHistoryChange.latestEvent),
                Event.SettingsUpdate(settingsListener.settings),
                Event.NewLocation(locationInfoCache.location),
                Event.WireGuardKeyStatus(keyStatusListener.keyStatus),
                Event.SplitTunnelingUpdate(splitTunneling.onChange.latestEvent),
                Event.CurrentVersion(appVersionInfoCache.currentVersion),
                Event.AppVersionInfo(appVersionInfoCache.appVersionInfo),
                Event.NewRelayList(relayListListener.relayList),
                Event.AuthToken(authTokenCache.authToken),
                Event.ListenerReady
            ).let { initialEvents ->
                if (vpnPermission.waitingForResponse) {
                    initialEvents + Event.VpnPermissionRequest
                } else {
                    initialEvents
                }
            }.forEach { event ->
                listener.send(event.message)
            }
        }
    }
}
