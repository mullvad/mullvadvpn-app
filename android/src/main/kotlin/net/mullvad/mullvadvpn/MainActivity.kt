package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.ComponentName
import android.content.Intent
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import android.support.v4.app.FragmentActivity

import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

class MainActivity : FragmentActivity() {
    var asyncDaemon = CompletableDeferred<MullvadDaemon>()
    val daemon
        get() = runBlocking { asyncDaemon.await() }

    var asyncRelayList: Deferred<RelayList> = fetchRelayList()
        private set
    val relayList: RelayList
        get() = runBlocking { asyncRelayList.await() }

    var asyncSettings = fetchSettings()
        private set
    val settings
        get() = runBlocking { asyncSettings.await() }

    val accountCache = AccountCache(this)

    var selectedRelayItem: RelayItem? = null

    private val restoreSelectedRelayListItemJob = restoreSelectedRelayListItem()
    private var waitForDaemonJob: Job? = null

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            val localBinder = binder as MullvadVpnService.LocalBinder

            waitForDaemonJob = GlobalScope.launch(Dispatchers.Default) {
                asyncDaemon.complete(localBinder.asyncDaemon.await())
            }
        }

        override fun onServiceDisconnected(className: ComponentName) {
            asyncDaemon.cancel()
            asyncDaemon = CompletableDeferred<MullvadDaemon>()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    override fun onStart() {
        super.onStart()

        val intent = Intent(this, MullvadVpnService::class.java)

        startService(intent)
        bindService(intent, serviceConnection, 0)
    }

    override fun onStop() {
        unbindService(serviceConnection)

        super.onStop()
    }

    override fun onDestroy() {
        accountCache.onDestroy()

        restoreSelectedRelayListItemJob.cancel()
        waitForDaemonJob?.cancel()
        asyncSettings.cancel()
        asyncRelayList.cancel()
        asyncDaemon.cancel()

        super.onDestroy()
    }

    fun openSettings() {
        supportFragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SettingsFragment())
            addToBackStack(null)
            commit()
        }
    }

    fun refetchSettings() {
        if (asyncSettings.isCompleted) {
            asyncSettings = fetchSettings()
            accountCache.settings = asyncSettings
        }
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun fetchRelayList() = GlobalScope.async(Dispatchers.Default) {
        RelayList(asyncDaemon.await().getRelayLocations())
    }

    private fun fetchSettings() = GlobalScope.async(Dispatchers.Default) {
        asyncDaemon.await().getSettings()
    }

    private fun restoreSelectedRelayListItem() = GlobalScope.launch(Dispatchers.Default) {
        val relaySettings = asyncSettings.await().relaySettings

        when (relaySettings) {
            is RelaySettings.CustomTunnelEndpoint -> selectedRelayItem = null
            is RelaySettings.RelayConstraints -> {
                val location = relaySettings.location
                val relayList = asyncRelayList.await()

                selectedRelayItem = relayList.findItemForLocation(location, true)
            }
        }
    }
}
