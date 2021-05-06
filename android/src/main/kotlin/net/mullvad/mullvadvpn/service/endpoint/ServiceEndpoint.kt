package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.persistence.SplitTunnelingPersistence
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.ConnectivityListener

class ServiceEndpoint(
    looper: Looper,
    internal val intermittentDaemon: Intermittent<MullvadDaemon>,
    val connectivityListener: ConnectivityListener,
    context: Context
) {
    private val listeners = mutableMapOf<Int, Messenger>()
    private val registrationQueue: SendChannel<Messenger> = startRegistrator()

    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    private var listenerIdCounter = 0

    val messenger = Messenger(dispatcher)

    val vpnPermission = VpnPermission(context, this)

    val connectionProxy = ConnectionProxy(vpnPermission, this)
    val settingsListener = SettingsListener(this)

    val accountCache = AccountCache(this)
    val appVersionInfoCache = AppVersionInfoCache(this)
    val authTokenCache = AuthTokenCache(this)
    val customDns = CustomDns(this)
    val keyStatusListener = KeyStatusListener(this)
    val locationInfoCache = LocationInfoCache(this)
    val relayListListener = RelayListListener(this)
    val splitTunneling = SplitTunneling(SplitTunnelingPersistence(context), this)
    val voucherRedeemer = VoucherRedeemer(this)

    init {
        dispatcher.registerHandler(Request.RegisterListener::class) { request ->
            registrationQueue.sendBlocking(request.listener)
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
        registrationQueue.close()

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
            val deadListeners = mutableSetOf<Int>()

            for ((id, listener) in listeners) {
                try {
                    listener.send(event.message)
                } catch (_: DeadObjectException) {
                    deadListeners.add(id)
                }
            }

            deadListeners.forEach { listeners.remove(it) }
        }
    }

    private fun startRegistrator() = GlobalScope.actor<Messenger>(
        Dispatchers.Default,
        Channel.UNLIMITED
    ) {
        try {
            while (true) {
                val listener = channel.receive()

                intermittentDaemon.await()

                registerListener(listener)
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Registration queue closed; stop registrator
        }
    }

    private fun registerListener(listener: Messenger) {
        synchronized(this) {
            val listenerId = newListenerId()

            listeners.put(listenerId, listener)

            val initialEvents = mutableListOf(
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
                Event.ListenerReady(messenger, listenerId)
            )

            if (vpnPermission.waitingForResponse) {
                initialEvents.add(Event.VpnPermissionRequest)
            }

            initialEvents.forEach { event ->
                listener.send(event.message)
            }
        }
    }

    private fun newListenerId(): Int {
        val listenerId = listenerIdCounter

        listenerIdCounter += 1

        return listenerId
    }
}
