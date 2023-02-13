package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.FragmentActivity
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.AdvancedSettingScreen
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.fragment.SplitTunnelingFragment
import net.mullvad.mullvadvpn.viewmodel.AdvancedSettingViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class AdvancedSettingFragment : BaseFragment() {

    private val advancesSettingViewModel by viewModel<AdvancedSettingViewModel>()

    @OptIn(ExperimentalMaterialApi::class)
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {

        //
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AdvancedSettingScreen(
                    uiState = advancesSettingViewModel.uiState.collectAsState().value,

                    onMtuChanged = { advancesSettingViewModel.onMtuChanged(it) },
                    onMtuSubmit = { advancesSettingViewModel.onSubmitMtu() },
                    onMtuFocusChanged = { advancesSettingViewModel.onMtuFocusChanged(it) },

                    onNavigateCellClicked = { onNavigationCellClicked(requireActivity()) },

                    onDnsCellClicked = { advancesSettingViewModel.setEditDnsIndex(it) },
                    onDnsCellLostFocus = {
                        advancesSettingViewModel.indexLostFocus(it)
                    },
                    onToggleCustomDns = { advancesSettingViewModel.toggleCustomDns(it) },
                    onConfirmDns = { index, item ->
                        advancesSettingViewModel.confirmDns(
                            index,
                            item
                        )
                    },
                    onRemoveDns = { index ->
                        advancesSettingViewModel.removeDnsClicked(index)
                    },
                    onDnsTextChanged = { index, item ->
                        advancesSettingViewModel.dnsChanged(
                            index,
                            item
                        )
                    },
                    onConfirmAddLocalDns = { advancesSettingViewModel.onConfirmAddLocalDns() },
                    onCancelLocalDns = { advancesSettingViewModel.onCancelLocalDns() },
                    onBackClick = { activity?.onBackPressed() },
                )
            }
        }
    }
    private fun onNavigationCellClicked(fragmentActivity: FragmentActivity) {
        val fragment =
            SplitTunnelingFragment::class.java.getConstructor()
                .newInstance()

        fragmentActivity.supportFragmentManager
            ?.beginTransaction()
            ?.apply {
                setCustomAnimations(
                    R.anim.fragment_enter_from_right,
                    R.anim.fragment_exit_to_left,
                    R.anim.fragment_half_enter_from_left,
                    R.anim.fragment_exit_to_right
                )
                replace(R.id.main_fragment, fragment)
                addToBackStack(null)
                commitAllowingStateLoss()
            }
    }
}
