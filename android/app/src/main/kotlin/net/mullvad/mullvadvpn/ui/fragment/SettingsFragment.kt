package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.SettingsScreen
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class SettingsFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {
    private val vm by viewModel<SettingsViewModel>()

    @OptIn(ExperimentalMaterialApi::class)
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    SettingsScreen(
                        uiState = state as SettingsUiState,
                        onVpnSettingCellClick = { openVpnSettingsFragment() },
                        onSplitTunnelingCellClick = { openSplitTunnelingFragment() },
                        onReportProblemCellClick = { openReportProblemFragment() },
                        onBackClick = { activity?.onBackPressed() }
                    )
                }
            }
        }
    }

    private fun openFragment(fragment: Fragment) {
        parentFragmentManager.beginTransaction().apply {
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

    private fun openVpnSettingsFragment() {
        openFragment(VpnSettingsFragment())
    }

    private fun openSplitTunnelingFragment() {
        openFragment(SplitTunnelingFragment())
    }

    private fun openReportProblemFragment() {
        openFragment(ProblemReportFragment())
    }
}
