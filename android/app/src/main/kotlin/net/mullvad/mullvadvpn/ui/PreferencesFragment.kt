package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import org.koin.android.ext.android.inject

class PreferencesFragment : BaseFragment() {

    // Injected dependencies
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private lateinit var allowLanToggle: ToggleCell
    private lateinit var autoConnectToggle: ToggleCell
    private lateinit var titleController: CollapsibleTitleController

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.preferences, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            requireMainActivity().onBackPressed()
        }

        allowLanToggle = view.findViewById<ToggleCell>(R.id.allow_lan).apply {
            listener = { state ->
                serviceConnectionManager.settingsListener()?.allowLan = when (state) {
                    CellSwitch.State.ON -> true
                    else -> false
                }
            }
        }

        autoConnectToggle = view.findViewById<ToggleCell>(R.id.auto_connect).apply {
            listener = { state ->
                serviceConnectionManager.settingsListener()?.autoConnect = when (state) {
                    CellSwitch.State.ON -> true
                    else -> false
                }
            }
        }

        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onDestroyView() {
        titleController.onDestroy()
        super.onDestroyView()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchSettingsSubscription()
        }
    }

    private fun CoroutineScope.launchSettingsSubscription() = launch {
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .flatMapLatest {
                callbackFlowFromNotifier(it.settingsListener.settingsNotifier)
            }
            .collect { settings ->
                if (settings != null) {
                    updateUi(settings)
                }
            }
    }

    private fun updateUi(settings: Settings) {
        jobTracker.newUiJob("updateUi") {
            val allowLanState = boolToSwitchState(settings.allowLan)
            val autoConnectState = boolToSwitchState(settings.autoConnect)

            if (isVisible) {
                allowLanToggle.state = allowLanState
                autoConnectToggle.state = autoConnectState
            } else {
                allowLanToggle.forcefullySetState(allowLanState)
                autoConnectToggle.forcefullySetState(autoConnectState)
            }
        }
    }

    private fun boolToSwitchState(pref: Boolean): CellSwitch.State {
        if (pref) {
            return CellSwitch.State.ON
        } else {
            return CellSwitch.State.OFF
        }
    }
}
