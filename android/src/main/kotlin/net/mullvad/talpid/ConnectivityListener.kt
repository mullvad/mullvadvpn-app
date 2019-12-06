package net.mullvad.talpid

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.NetworkInfo
import android.net.NetworkInfo.DetailedState
import net.mullvad.talpid.util.EventNotifier

class ConnectivityListener : BroadcastReceiver() {
    val connectivityNotifier = EventNotifier(true)

    var isConnected = true
        private set(value) {
            field = value

            if (senderAddress != 0L) {
                notifyConnectivityChange(value, senderAddress)
            }

            connectivityNotifier.notify(value)
        }

    var senderAddress = 0L

    fun register(context: Context) {
        val intentFilter = IntentFilter()

        intentFilter.addAction(ConnectivityManager.CONNECTIVITY_ACTION)
        context.registerReceiver(this, intentFilter)

        checkConnectionState(context)
    }

    fun unregister(context: Context) {
        context.unregisterReceiver(this)
    }

    override fun onReceive(context: Context, intent: Intent) {
        val networkInfo =
            intent.getParcelableExtra<NetworkInfo>(ConnectivityManager.EXTRA_NETWORK_INFO)

        if (networkInfo == null) {
            checkConnectionState(context)
        } else if (networkInfo.type != ConnectivityManager.TYPE_VPN) {
            if (networkInfo.detailedState == DetailedState.DISCONNECTED) {
                checkConnectionState(context)
            } else if (networkInfo.detailedState == DetailedState.CONNECTED) {
                isConnected = true
            }
        }
    }

    private fun checkConnectionState(context: Context) {
        val connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        isConnected = connectivityManager.allNetworks
            .map({ network -> connectivityManager.getNetworkInfo(network) })
            .filterNotNull()
            .any({ networkInfo ->
                networkInfo.type != ConnectivityManager.TYPE_VPN &&
                    networkInfo.detailedState == DetailedState.CONNECTED
            })
    }

    private fun finalize() {
        destroySender(senderAddress)
        senderAddress = 0L
    }

    private external fun notifyConnectivityChange(isConnected: Boolean, senderAddress: Long)
    private external fun destroySender(senderAddress: Long)
}
