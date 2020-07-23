package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.widget.Cell

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var wireguardMtuInput: CellInput
    private lateinit var titleController: CollapsibleTitleController

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
            jobTracker.newBackgroundJob("updateMtu") {
                daemon.setWireguardMtu(mtu)
            }
        }

        view.findViewById<TextView>(R.id.wireguard_mtu_footer).apply {
            text = context.getString(R.string.wireguard_mtu_footer, MIN_MTU_VALUE, MAX_MTU_VALUE)
        }

        view.findViewById<Cell>(R.id.wireguard_keys).onClickListener = {
            openSubFragment(WireguardKeyFragment())
        }

        view.findViewById<Cell>(R.id.split_tunnelling).onClickListener = {
            openSubFragment(SplitTunnellingFragment())
        }

        settingsListener.subscribe(this) { settings ->
            updateUi(settings)
        }

        titleController = CollapsibleTitleController(view)

        return view
    }

    private fun updateUi(settings: Settings) {
        jobTracker.newUiJob("updateUi") {
            if (!wireguardMtuInput.hasFocus) {
                wireguardMtuInput.value = settings.tunnelOptions.wireguard.mtu
            }
        }
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
        settingsListener.unsubscribe(this)
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
