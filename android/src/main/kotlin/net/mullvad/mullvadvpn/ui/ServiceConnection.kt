package net.mullvad.mullvadvpn.ui

import android.os.Looper
import android.os.Messenger
import android.os.RemoteException
import android.util.Log
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.KeyStatusListener
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.SettingsListener
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.util.DispatchingHandler

class ServiceConnection(private val service: ServiceInstance) {
    val dispatcher = DispatchingHandler(Looper.getMainLooper()) { message ->
        Event.fromMessage(message)
    }

    val daemon = service.daemon
    val accountCache = AccountCache(service.messenger, dispatcher)
    val connectionProxy = ConnectionProxy(service.messenger, dispatcher)
    val keyStatusListener = KeyStatusListener(service.messenger, dispatcher)
    val locationInfoCache = LocationInfoCache(dispatcher)
    val settingsListener = SettingsListener(dispatcher)
    val splitTunneling = SplitTunneling(service.messenger, dispatcher)

    val appVersionInfoCache = AppVersionInfoCache(dispatcher, settingsListener)
    val customDns = CustomDns(service.messenger, settingsListener)
    var relayListListener = RelayListListener(dispatcher, settingsListener)

    init {
        registerListener()
    }

    fun onDestroy() {
        dispatcher.onDestroy()

        accountCache.onDestroy()
        connectionProxy.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        settingsListener.onDestroy()

        appVersionInfoCache.onDestroy()
        customDns.onDestroy()
        relayListListener.onDestroy()
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
