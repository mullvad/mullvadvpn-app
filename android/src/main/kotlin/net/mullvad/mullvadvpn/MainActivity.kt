package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.support.v4.app.FragmentActivity

import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

class MainActivity : FragmentActivity() {
    val activityCreated = CompletableDeferred<Unit>()

    val asyncDaemon = startDaemon()
    val daemon
        get() = runBlocking { asyncDaemon.await() }

    var asyncRelayList: Deferred<RelayList> = fetchRelayList()
        private set
    val relayList: RelayList
        get() = runBlocking { asyncRelayList.await() }

    var selectedRelayItem: RelayItem? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        activityCreated.complete(Unit)

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    override fun onDestroy() {
        asyncRelayList.cancel()
        asyncDaemon.cancel()

        super.onDestroy()
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun startDaemon() = GlobalScope.async(Dispatchers.Default) {
        activityCreated.await()
        ApiRootCaFile().extract(this@MainActivity)
        MullvadDaemon()
    }

    private fun fetchRelayList() = GlobalScope.async(Dispatchers.Default) {
        RelayList(asyncDaemon.await().getRelayLocations())
    }
}
