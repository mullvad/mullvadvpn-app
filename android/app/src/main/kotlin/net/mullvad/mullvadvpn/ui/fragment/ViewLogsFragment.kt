package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.ViewLogsScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class ViewLogsFragment : BaseFragment() {
    private val vm by viewModel<ViewLogsViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {

        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val uiState = vm.uiState.collectAsState()
                    ViewLogsScreen(
                        uiState = uiState.value,
                        onBackClick = { activity?.onBackPressed() }
                    )
                }
            }
        }
    }
}
