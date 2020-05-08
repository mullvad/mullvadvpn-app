package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R

class OutOfTimeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ) = inflater.inflate(R.layout.out_of_time, container, false)
}
