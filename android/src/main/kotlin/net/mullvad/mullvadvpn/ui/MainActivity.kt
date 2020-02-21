package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.content.ComponentName
import android.content.Intent
import android.os.Bundle
import android.os.IBinder
import android.support.v4.app.FragmentActivity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.talpid.util.EventNotifier

class MainActivity : FragmentActivity() {
    companion object {
        val KEY_SHOULD_CONNECT = "should_connect"
    }

    val problemReport = MullvadProblemReport()
    val serviceNotifier = EventNotifier<ServiceConnection?>(null)

    private var service: MullvadVpnService.LocalBinder? = null
    private var serviceConnection: ServiceConnection? = null
    private var serviceConnectionSubscription: Int? = null
    private var shouldConnect = false

    private val serviceConnectionManager = object : android.content.ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            val localBinder = binder as MullvadVpnService.LocalBinder

            service = localBinder

            serviceConnectionSubscription = localBinder.serviceNotifier.subscribe { service ->
                serviceConnection?.onDestroy()

                val newConnection = service?.let { safeService ->
                    ServiceConnection(safeService, this@MainActivity)
                }

                serviceConnection = newConnection
                serviceNotifier.notify(newConnection)

                if (shouldConnect) {
                    tryToConnect()
                }
            }
        }

        override fun onServiceDisconnected(className: ComponentName) {
            serviceConnectionSubscription?.let { subscriptionId ->
                service?.apply {
                    serviceNotifier.unsubscribe(subscriptionId)
                }
            }

            serviceConnection = null
            serviceConnectionSubscription = null
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
            shouldConnect = true
            tryToConnect()
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
        unbindService(serviceConnectionManager)

        super.onStop()
    }

    override fun onDestroy() {
        serviceNotifier.unsubscribeAll()
        serviceConnection?.onDestroy()

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

    fun returnToLaunchScreen() {
        supportFragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    fun requestVpnPermission(intent: Intent) {
        startActivityForResult(intent, 0)
    }

    fun quit() {
        service?.stop()
        finishAndRemoveTask()
    }

    private fun tryToConnect() {
        serviceConnection?.apply {
            connectionProxy.connect()
            shouldConnect = false
        }
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun setVpnPermission(allow: Boolean) = GlobalScope.launch(Dispatchers.Default) {
        serviceConnection?.connectionProxy?.vpnPermission?.complete(allow)
    }
}
