package net.mullvad.mullvadvpn.service.endpoint

import android.content.Context
import android.os.Looper
import android.os.Messenger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.persistence.SplitTunnelingPersistence
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.mullvadvpn.util.trySendEvent
import net.mullvad.talpid.ConnectivityListener

const val SHOULD_LOG_DEAD_OBJECT_EXCEPTION = true

class ServiceEndpoint(
    looper: Looper,
    internal val intermittentDaemon: Intermittent<MullvadDaemon>,
    val connectivityListener: ConnectivityListener,
    context: Context
) {
    companion object {
        sealed class Command {
            data class RegisterListener(val listener: Messenger) : Command()
            data class UnregisterListener(val listenerId: Int) : Command()
        }
    }

    private val listeners = mutableMapOf<Int, Messenger>()
    private val commands: SendChannel<Command> = startRegistrator()

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
    val locationInfoCache = LocationInfoCache(this)
    val relayListListener = RelayListListener(this)
    val splitTunneling = SplitTunneling(SplitTunnelingPersistence(context), this)
    val voucherRedeemer = VoucherRedeemer(this)

    private val deviceRepositoryBackend = DaemonDeviceDataSource(this)

    init {
        dispatcher.apply {
            registerHandler(Request.RegisterListener::class) { request ->
                commands.trySendBlocking(Command.RegisterListener(request.listener))
            }

            registerHandler(Request.UnregisterListener::class) { request ->
                commands.trySendBlocking(Command.UnregisterListener(request.listenerId))
            }
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
        commands.close()

        accountCache.onDestroy()
        appVersionInfoCache.onDestroy()
        authTokenCache.onDestroy()
        connectionProxy.onDestroy()
        customDns.onDestroy()
        deviceRepositoryBackend.onDestroy()
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
                if (!listener.trySendEvent(
                        event,
                        SHOULD_LOG_DEAD_OBJECT_EXCEPTION
                    )
                ) {
                    deadListeners.add(id)
                }
            }
            deadListeners.forEach { listeners.remove(it) }
        }
    }

    private fun startRegistrator() = GlobalScope.actor<Command>(
        Dispatchers.Default,
        Channel.UNLIMITED
    ) {
        try {
            for (command in channel) {
                when (command) {
                    is Command.RegisterListener -> {
                        intermittentDaemon.await()

                        registerListener(command.listener)
                    }
                    is Command.UnregisterListener -> unregisterListener(command.listenerId)
                }
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
                Event.AccountHistoryEvent(accountCache.onAccountHistoryChange.latestEvent),
                Event.SettingsUpdate(settingsListener.settings),
                Event.NewLocation(locationInfoCache.location),
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

            val didSuccessfullySendAllMessages = initialEvents.all { event ->
                listener.trySendEvent(
                    event,
                    SHOULD_LOG_DEAD_OBJECT_EXCEPTION
                )
            }
            if (didSuccessfullySendAllMessages.not()) {
                listeners.remove(listenerId)
            }
        }
    }

    private fun unregisterListener(listenerId: Int) {
        synchronized(this) {
            listeners.remove(listenerId)
        }
    }

    private fun newListenerId(): Int {
        val listenerId = listenerIdCounter

        listenerIdCounter += 1

        return listenerId
    }
}
