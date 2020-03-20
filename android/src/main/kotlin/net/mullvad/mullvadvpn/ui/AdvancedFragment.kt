package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var wireguardMtuInput: CellInput

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

        return view
    }
}
