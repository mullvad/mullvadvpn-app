package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.SelectLocationScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class SelectLocationFragment : BaseFragment() {

    private val vm by viewModel<SelectLocationViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    SelectLocationScreen(
                        uiState = state,
                        uiCloseAction = vm.uiCloseAction,
                        enterTransitionEndAction = vm.enterTransitionEndAction,
                        onSelectRelay = vm::selectRelay,
                        onSearchTermInput = vm::onSearchTermInput,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() },
                        removeOwnershipFilter = vm::removeOwnerFilter,
                        removeProviderFilter = vm::removeProviderFilter,
                        onFilterClick = ::openFilterView
                    )
                }
            }
        }
    }

    private fun openFilterView() {
        (context as? MainActivity)?.openFilter()
    }

    override fun onEnterTransitionAnimationEnd() {
        vm.onTransitionAnimationEnd()
    }
}
