package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.DeviceRevokedScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class DeviceRevokedFragment : Fragment() {
    private val deviceRevokedViewModel: DeviceRevokedViewModel by viewModel()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = deviceRevokedViewModel.uiState.collectAsState().value
                    DeviceRevokedScreen(
                        state = state,
                        onSettingsClicked = this@DeviceRevokedFragment::openSettingsView,
                        onGoToLoginClicked = deviceRevokedViewModel::onGoToLoginClicked
                    )
                }
            }
        }
    }

    private fun openSettingsView() {
        (context as? MainActivity)?.openSettings()
    }
}
