package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.SelectLocationScreen
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class SelectLocationFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {

    private val vm by viewModel<SelectLocationViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val systemUiController = rememberSystemUiController()
                    systemUiController.setStatusBarColor(MaterialTheme.colorScheme.background)
                    systemUiController.setNavigationBarColor(MaterialTheme.colorScheme.background)

                    val state = vm.uiState.collectAsState().value
                    SelectLocationScreen(
                        uiState = state,
                        uiCloseAction = vm.uiCloseAction,
                        onSelectRelay = vm::selectRelay,
                        onSearchTermInput = vm::onSearchTermInput,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() }
                    )
                }
            }
        }
    }
}
