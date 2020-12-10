package net.mullvad.mullvadvpn.ui

import android.os.Looper
import android.os.Messenger
import android.os.RemoteException
import android.util.Log
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.mullvadvpn.ui.serviceconnection.SettingsListener
import net.mullvad.mullvadvpn.util.DispatchingHandler

class ServiceConnection(private val service: ServiceInstance, val mainActivity: MainActivity) {
    val dispatcher = DispatchingHandler(Looper.getMainLooper()) { message ->
        Event.fromMessage(message)
    }

    val daemon = service.daemon
    val accountCache = service.accountCache
    val connectionProxy = service.connectionProxy
    val customDns = service.customDns
    val keyStatusListener = service.keyStatusListener
    val locationInfoCache = service.locationInfoCache
    val settingsListener = SettingsListener(dispatcher)
    val splitTunneling = service.splitTunneling

    val appVersionInfoCache = AppVersionInfoCache(mainActivity, daemon, settingsListener)
    var relayListListener = RelayListListener(daemon, settingsListener)

    init {
        appVersionInfoCache.onCreate()
        connectionProxy.mainActivity = mainActivity
        registerListener()
    }

    fun onDestroy() {
        dispatcher.onDestroy()

        settingsListener.onDestroy()

        appVersionInfoCache.onDestroy()
        relayListListener.onDestroy()
        connectionProxy.mainActivity = null
    }

    private fun registerListener() {
        val listener = Messenger(dispatcher)
        val request = Request.RegisterListener(listener)

        try {
            service.messenger.send(request.message)
        } catch (exception: RemoteException) {
            Log.e("mullvad", "Failed to register listener for service events", exception)
        }
    }
}
