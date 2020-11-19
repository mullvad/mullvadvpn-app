package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.widget.MtuCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var wireguardMtuInput: MtuCell
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

        wireguardMtuInput = view.findViewById<MtuCell>(R.id.wireguard_mtu).apply {
            onSubmit = { mtu ->
                jobTracker.newBackgroundJob("updateMtu") {
                    daemon.setWireguardMtu(mtu)
                }
            }
        }

        view.findViewById<NavigateCell>(R.id.wireguard_keys).apply {
            targetFragment = WireguardKeyFragment::class
        }

        view.findViewById<NavigateCell>(R.id.split_tunneling).apply {
            targetFragment = SplitTunnelingFragment::class
        }

        settingsListener.subscribe(this) { settings ->
            updateUi(settings)
        }

        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
        settingsListener.unsubscribe(this)
    }

    private fun updateUi(settings: Settings) {
        jobTracker.newUiJob("updateUi") {
            if (!wireguardMtuInput.hasFocus) {
                wireguardMtuInput.value = settings.tunnelOptions.wireguard.mtu
            }
        }
    }
}
