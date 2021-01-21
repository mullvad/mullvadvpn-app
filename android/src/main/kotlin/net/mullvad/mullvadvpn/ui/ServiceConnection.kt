package net.mullvad.mullvadvpn.ui

import android.os.Looper
import android.os.Messenger
import android.os.RemoteException
import android.util.Log
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.KeyStatusListener
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.SettingsListener
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import net.mullvad.mullvadvpn.util.DispatchingHandler

class ServiceConnection(connection: Messenger) {
    val dispatcher = DispatchingHandler(Looper.getMainLooper()) { message ->
        Event.fromMessage(message)
    }

    val accountCache = AccountCache(connection, dispatcher)
    val authTokenCache = AuthTokenCache(connection, dispatcher)
    val connectionProxy = ConnectionProxy(connection, dispatcher)
    val keyStatusListener = KeyStatusListener(connection, dispatcher)
    val locationInfoCache = LocationInfoCache(dispatcher)
    val settingsListener = SettingsListener(connection, dispatcher)
    val splitTunneling = SplitTunneling(connection, dispatcher)
    val voucherRedeemer = VoucherRedeemer(connection, dispatcher)

    val appVersionInfoCache = AppVersionInfoCache(dispatcher, settingsListener)
    val customDns = CustomDns(connection, settingsListener)
    var relayListListener = RelayListListener(connection, dispatcher, settingsListener)

    init {
        registerListener(connection)
    }

    fun onDestroy() {
        dispatcher.onDestroy()

        accountCache.onDestroy()
        authTokenCache.onDestroy()
        connectionProxy.onDestroy()
        keyStatusListener.onDestroy()
        locationInfoCache.onDestroy()
        settingsListener.onDestroy()
        voucherRedeemer.onDestroy()

        appVersionInfoCache.onDestroy()
        customDns.onDestroy()
        relayListListener.onDestroy()
    }

    private fun registerListener(connection: Messenger) {
        val listener = Messenger(dispatcher)
        val request = Request.RegisterListener(listener)

        try {
            connection.send(request.message)
        } catch (exception: RemoteException) {
            Log.e("mullvad", "Failed to register listener for service events", exception)
        }
    }
}
