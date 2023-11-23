package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.FilterScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.FilterViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class FilterFragment : Fragment() {

    private val vm by viewModel<FilterViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    FilterScreen(
                        uiState = state,
                        onSelectedOwnership = vm::setSelectedOwnership,
                        onAllProviderCheckChange = vm::setAllProviders,
                        onSelectedProviders = vm::setSelectedProvider,
                        uiCloseAction = vm.uiCloseAction,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() },
                        onApplyClick = vm::onApplyButtonClicked,
                    )
                }
            }
        }
    }
}
