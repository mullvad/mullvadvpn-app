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

class PreferencesFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var allowLanToggle: CellSwitch

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
            settingsListener.settings?.let { settings ->
                if (settings.allowLan) {
                    forcefullySetState(CellSwitch.State.ON)
                } else {
                    forcefullySetState(CellSwitch.State.OFF)
                }
            }

            listener = { state ->
                when (state) {
                    CellSwitch.State.ON -> daemon.setAllowLan(true)
                    CellSwitch.State.OFF -> daemon.setAllowLan(false)
                }
            }
        }

        settingsListener.onAllowLanChange = { allowLan ->
            updateUiJob?.cancel()
            updateUiJob = updateUi(allowLan)
        }

        return view
    }

    override fun onSafelyDestroyView() {
        settingsListener.onAllowLanChange = null
    }

    private fun updateUi(allowLan: Boolean) = GlobalScope.launch(Dispatchers.Main) {
        if (allowLan) {
            allowLanToggle.state = CellSwitch.State.ON
        } else {
            allowLanToggle.state = CellSwitch.State.OFF
        }
    }
}
