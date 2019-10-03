package net.mullvad.talpid

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.NetworkInfo
import android.net.NetworkInfo.DetailedState

class ConnectivityListener : BroadcastReceiver() {
    var isConnected = true
        private set

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

        if (networkInfo.type != ConnectivityManager.TYPE_VPN) {
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
            .any({ networkInfo ->
                networkInfo.type != ConnectivityManager.TYPE_VPN &&
                    networkInfo.detailedState == DetailedState.CONNECTED
            })
    }
}
