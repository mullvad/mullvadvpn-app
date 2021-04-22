package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.app.UiModeManager
import android.content.ComponentName
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.res.Configuration
import android.net.VpnService
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.view.WindowManager
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentActivity
import androidx.fragment.app.FragmentManager
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection
import net.mullvad.talpid.util.EventNotifier

open class MainActivity : FragmentActivity() {
    val problemReport = MullvadProblemReport()
    val serviceNotifier = EventNotifier<ServiceConnection?>(null)

    private var isUiVisible = false
    private var service: MullvadVpnService.LocalBinder? = null
    private var serviceConnection: ServiceConnection? = null
    private var visibleSecureScreens = HashSet<Fragment>()

    private val deviceIsTv by lazy {
        val uiModeManager = getSystemService(UI_MODE_SERVICE) as UiModeManager

        uiModeManager.currentModeType == Configuration.UI_MODE_TYPE_TELEVISION
    }

    private val serviceConnectionManager = object : android.content.ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            android.util.Log.d("mullvad", "UI successfully connected to the service")
            val localBinder = binder as MullvadVpnService.LocalBinder

            service = localBinder

            localBinder.isUiVisible = isUiVisible

            localBinder.serviceNotifier.subscribe(this@MainActivity) { service ->
                android.util.Log.d("mullvad", "UI connection to the service changed: $service")
                serviceConnection?.onDestroy()

                serviceConnection = service?.let { safeService ->
                    ServiceConnection(safeService, ::handleNewServiceConnection).apply {
                        vpnPermission.onRequest = ::requestVpnPermission
                    }
                }

                if (service == null) {
                    serviceNotifier.notify(null)
                }
            }
        }

        override fun onServiceDisconnected(className: ComponentName) {
            android.util.Log.d("mullvad", "UI lost the connection to the service")
            service?.serviceNotifier?.unsubscribe(this@MainActivity)
            serviceConnection?.onDestroy()
            service = null
            serviceConnection = null
            serviceNotifier.notify(null)
        }
    }

    var backButtonHandler: (() -> Boolean)? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        if (deviceIsTv) {
            setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE)
        }

        super.onCreate(savedInstanceState)

        problemReport.apply {
            logDirectory.complete(filesDir)
            cacheDirectory.complete(cacheDir)
        }

        setContentView(R.layout.main)

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    override fun onStart() {
        android.util.Log.d("mullvad", "Starting main activity")
        super.onStart()

        isUiVisible = true

        val intent = Intent(this, MullvadVpnService::class.java)

        if (Build.VERSION.SDK_INT >= 26) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }

        bindService(intent, serviceConnectionManager, 0)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        serviceConnection?.vpnPermission?.grant(resultCode == Activity.RESULT_OK)
    }

    override fun onBackPressed() {
        val handled = backButtonHandler?.invoke() ?: false

        if (!handled) {
            super.onBackPressed()
        }
    }

    override fun onStop() {
        android.util.Log.d("mullvad", "Stoping main activity")
        isUiVisible = false
        service?.isUiVisible = false
        service = null
        unbindService(serviceConnectionManager)

        super.onStop()
    }

    override fun onDestroy() {
        serviceNotifier.unsubscribeAll()
        serviceConnection?.onDestroy()

        super.onDestroy()
    }

    fun enterSecureScreen(screen: Fragment) {
        synchronized(this) {
            visibleSecureScreens.add(screen)

            if (!BuildConfig.DEBUG) {
                window?.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
    }

    fun leaveSecureScreen(screen: Fragment) {
        synchronized(this) {
            visibleSecureScreens.remove(screen)

            if (!BuildConfig.DEBUG && visibleSecureScreens.isEmpty()) {
                window?.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
    }

    fun openSettings() {
        supportFragmentManager.beginTransaction().apply {
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
        supportFragmentManager.apply {
            popBackStack(null, FragmentManager.POP_BACK_STACK_INCLUSIVE)

            beginTransaction().apply {
                replace(R.id.main_fragment, LaunchFragment())
                commit()
            }
        }
    }

    private fun handleNewServiceConnection(connection: ServiceConnection) {
        serviceNotifier.notify(connection)
    }

    @Suppress("DEPRECATION")
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)

        startActivityForResult(intent, 0)
    }

    private fun addInitialFragment() {
        supportFragmentManager.beginTransaction().apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }
}
