package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var wireguardMtuInput: CellInput
    private lateinit var wireguardKeysMenu: View

    private var updateUiJob: Job? = null

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.advanced, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        wireguardMtuInput =
            CellInput(view.findViewById(R.id.wireguard_mtu_input), MIN_MTU_VALUE, MAX_MTU_VALUE)

        wireguardMtuInput.onSubmit = { mtu ->
            GlobalScope.launch(Dispatchers.Default) {
                daemon.setWireguardMtu(mtu)
            }
        }

        view.findViewById<TextView>(R.id.wireguard_mtu_footer).apply {
            text = context.getString(R.string.wireguard_mtu_footer, MIN_MTU_VALUE, MAX_MTU_VALUE)
        }

        wireguardKeysMenu = view.findViewById<View>(R.id.wireguard_keys).apply {
            setOnClickListener {
                openSubFragment(WireguardKeyFragment())
            }
        }

        settingsListener.subscribe(this) { settings ->
            updateUi(settings)
        }

        return view
    }

    private fun updateUi(settings: Settings) {
        updateUiJob?.cancel()
        updateUiJob = GlobalScope.launch(Dispatchers.Main) {
            if (!wireguardMtuInput.hasFocus) {
                wireguardMtuInput.value = settings.tunnelOptions.wireguard.mtu
            }
        }
    }

    override fun onSafelyDestroyView() {
        settingsListener.unsubscribe(this)
        updateUiJob?.cancel()
    }

    private fun openSubFragment(fragment: Fragment) {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_half_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, fragment)
            addToBackStack(null)
            commit()
        }
    }
}
