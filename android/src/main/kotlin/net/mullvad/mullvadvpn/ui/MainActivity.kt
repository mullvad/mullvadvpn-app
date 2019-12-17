package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.content.ComponentName
import android.content.Intent
import android.os.Bundle
import android.os.IBinder
import android.support.v4.app.FragmentActivity
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.talpid.util.EventNotifier

class MainActivity : FragmentActivity() {
    companion object {
        val KEY_SHOULD_CONNECT = "should_connect"
    }

    private var serviceConnection: ServiceConnection? = null
    private var serviceConnectionSubscription: Int? = null

    var service = CompletableDeferred<MullvadVpnService.LocalBinder>()
        private set

    val problemReport = MullvadProblemReport()
    val serviceNotifier = EventNotifier<ServiceConnection?>(null)

    val connectionProxy: ConnectionProxy
        get() = serviceConnection!!.connectionProxy

    private var quitJob: Job? = null
    private var serviceToStop: MullvadVpnService.LocalBinder? = null
    private var waitForDaemonJob: Job? = null

    private val serviceConnectionManager = object : android.content.ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            val localBinder = binder as MullvadVpnService.LocalBinder

            serviceConnectionSubscription = localBinder.serviceNotifier.subscribe { service ->
                serviceConnection?.onDestroy()

                val newConnection = service?.let { service ->
                    ServiceConnection(service, this@MainActivity)
                }

                serviceConnection = newConnection
                serviceNotifier.notify(newConnection)
            }

            waitForDaemonJob = GlobalScope.launch(Dispatchers.Default) {
                localBinder.resetComplete?.await()
                service.complete(localBinder)
            }
        }

        override fun onServiceDisconnected(className: ComponentName) {
            waitForDaemonJob?.cancel()
            waitForDaemonJob = null

            serviceConnectionSubscription?.let { subscription ->
                runBlocking {
                    service.await().serviceNotifier.unsubscribe(subscription)
                }
                serviceConnection = null
            }

            service.cancel()
            service = CompletableDeferred<MullvadVpnService.LocalBinder>()

            serviceNotifier.notify(null)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        if (savedInstanceState == null) {
            addInitialFragment()
        }

        if (intent.getBooleanExtra(KEY_SHOULD_CONNECT, false)) {
            connectionProxy.connect()
        }
    }

    override fun onStart() {
        super.onStart()

        val intent = Intent(this, MullvadVpnService::class.java)

        startService(intent)
        bindService(intent, serviceConnectionManager, 0)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        setVpnPermission(resultCode == Activity.RESULT_OK)
    }

    override fun onStop() {
        quitJob?.cancel()

        serviceNotifier.unsubscribeAll()

        serviceToStop?.apply { stop() }
        unbindService(serviceConnectionManager)

        super.onStop()
    }

    override fun onDestroy() {
        serviceConnection?.onDestroy()

        waitForDaemonJob?.cancel()

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

    fun requestVpnPermission(intent: Intent) {
        startActivityForResult(intent, 0)
    }

    fun quit() {
        quitJob?.cancel()
        quitJob = GlobalScope.launch(Dispatchers.Main) {
            serviceToStop = service.await()
            finishAndRemoveTask()
        }
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun setVpnPermission(allow: Boolean) = GlobalScope.launch(Dispatchers.Default) {
        connectionProxy.vpnPermission.complete(allow)
    }
}
