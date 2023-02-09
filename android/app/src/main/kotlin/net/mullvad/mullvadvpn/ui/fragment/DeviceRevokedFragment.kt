package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.colorResource
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.AppTheme
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.screen.DeviceRevokedScreen
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
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

                    val topColor = colorResource(
                        if (state == DeviceRevokedUiState.SECURED) {
                            R.color.green
                        } else {
                            R.color.red
                        }
                    )

                    ScaffoldWithTopBar(
                        topBarColor = topColor,
                        statusBarColor = topColor,
                        navigationBarColor = colorResource(id = R.color.darkBlue),
                        onSettingsClicked = this@DeviceRevokedFragment::openSettingsView,
                        content = { DeviceRevokedScreen(deviceRevokedViewModel) }
                    )
                }
            }
        }
    }

    private fun openSettingsView() {
        (context as? MainActivity)?.openSettings()
    }
}
