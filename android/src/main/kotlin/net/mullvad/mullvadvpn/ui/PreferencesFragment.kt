package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings

class PreferencesFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var allowLanToggle: CellSwitch
    private lateinit var autoConnectToggle: CellSwitch

    private var subscriptionId: Int? = null
    private var updateUiJob: Job? = null

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.preferences, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        allowLanToggle = view.findViewById<CellSwitch>(R.id.allow_lan_toggle).apply {
            val allowLan = settingsListener.settings.allowLan

            forcefullySetState(CellSwitch.State.fromBoolean(allowLan))

            listener = { state -> daemon.setAllowLan(state.isOn) }
        }

        autoConnectToggle = view.findViewById<CellSwitch>(R.id.auto_connect_toggle).apply {
            val autoConnect = settingsListener.settings.autoConnect

            forcefullySetState(CellSwitch.State.fromBoolean(autoConnect))

            listener = { state -> daemon.setAutoConnect(state.isOn) }
        }

        settingsListener.subscribe({ settings -> updateUi(settings) })

        return view
    }

    private fun updateUi(settings: Settings) {
        updateUiJob?.cancel()
        updateUiJob = GlobalScope.launch(Dispatchers.Main) {
            allowLanToggle.state = CellSwitch.State.fromBoolean(settings.allowLan)
            autoConnectToggle.state = CellSwitch.State.fromBoolean(settings.autoConnect)
        }
    }

    override fun onSafelyDestroyView() {
        subscriptionId?.let { id -> settingsListener.unsubscribe(id) }
    }
}
