package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.app.UiModeManager
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.res.Configuration
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentActivity
import androidx.fragment.app.FragmentManager
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.fragments.DeviceRevokedFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

open class MainActivity : FragmentActivity() {
    val problemReport = MullvadProblemReport()

    private var visibleSecureScreens = HashSet<Fragment>()

    private val deviceIsTv by lazy {
        val uiModeManager = getSystemService(UI_MODE_SERVICE) as UiModeManager

        uiModeManager.currentModeType == Configuration.UI_MODE_TYPE_TELEVISION
    }

    var backButtonHandler: (() -> Boolean)? = null

    private lateinit var serviceConnectionManager: ServiceConnectionManager
    private lateinit var deviceRepository: DeviceRepository

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(uiModule)

        getKoin().apply {
            serviceConnectionManager = get()
            deviceRepository = get()
        }

        requestedOrientation = if (deviceIsTv) {
            ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
        } else {
            ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
        }

        super.onCreate(savedInstanceState)

        problemReport.apply {
            logDirectory.complete(filesDir)
            cacheDirectory.complete(cacheDir)
        }

        setContentView(R.layout.main)

        launchDeviceStateHandler()
    }

    override fun onStart() {
        Log.d("mullvad", "Starting main activity")
        super.onStart()
        serviceConnectionManager.bind(vpnPermissionRequestHandler = ::requestVpnPermission)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        serviceConnectionManager.onVpnPermissionResult(resultCode == Activity.RESULT_OK)
    }

    override fun onBackPressed() {
        val handled = backButtonHandler?.invoke() ?: false

        if (!handled) {
            super.onBackPressed()
        }
    }

    override fun onStop() {
        Log.d("mullvad", "Stopping main activity")
        super.onStop()

        // NOTE: `super.onStop()` must be called before unbinding due to the fragment state handling
        // otherwise the fragments will believe there was an unexpected disconnect.
        serviceConnectionManager.unbind()
    }

    override fun onDestroy() {
        serviceConnectionManager.onDestroy()
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

    private fun launchDeviceStateHandler() {
        var currentState: DeviceState? = null

        lifecycleScope.launch {
            deviceRepository.deviceState
                .flowWithLifecycle(lifecycle, Lifecycle.State.RESUMED)
                .debounce {
                    // Debounce DeviceState.Unknown to delay view transitions during reconnect.
                    it.addDebounceForUnknownState()
                }
                .distinctByDeviceState()
                .filter { newState -> newState != currentState }
                .collect { newState ->
                    when (newState) {
                        is DeviceState.Initial,
                        is DeviceState.Unknown -> openLaunchView()
                        is DeviceState.LoggedOut -> openLoginView()
                        is DeviceState.Revoked -> openRevokedView()
                        is DeviceState.LoggedIn -> openConnectView()
                    }
                    currentState = newState
                }
        }
    }

    private fun Flow<DeviceState>.distinctByDeviceState(): Flow<DeviceState> {
        return this.distinctUntilChangedBy { it::class }
    }

    private fun DeviceState.addDebounceForUnknownState(): Long {
        return if (this is DeviceState.Unknown) {
            UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
        } else {
            ZERO_DEBOUNCE_DELAY_MILLISECONDS
        }
    }

    @Suppress("DEPRECATION")
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)

        startActivityForResult(intent, 0)
    }

    private fun openLaunchView() {
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun openConnectView() {
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }

    private fun openLoginView() {
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun openRevokedView() {
        supportFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, DeviceRevokedFragment())
            commit()
        }
    }

    companion object {
        private const val ZERO_DEBOUNCE_DELAY_MILLISECONDS = 0L
        private const val UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS = 2000L
    }
}
