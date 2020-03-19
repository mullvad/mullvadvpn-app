package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var enableIpv6Toggle: CellSwitch

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.advanced, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        enableIpv6Toggle = view.findViewById<CellSwitch>(R.id.enable_ipv6_toggle).apply {
            listener = { state ->
                when (state) {
                    CellSwitch.State.ON -> daemon.setEnableIpv6(true)
                    CellSwitch.State.OFF -> daemon.setEnableIpv6(false)
                }
            }
        }

        return view
    }
}
