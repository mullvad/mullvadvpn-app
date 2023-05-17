package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.SplitTunnelingScreen
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.di.APPS_SCOPE
import net.mullvad.mullvadvpn.di.SERVICE_CONNECTION_SCOPE
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.android.ext.android.getKoin
import org.koin.androidx.viewmodel.ViewModelOwner
import org.koin.androidx.viewmodel.scope.viewModel
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope

class SplitTunnelingFragment : BaseFragment() {
    private val scope: Scope =
        getKoin().getOrCreateScope(APPS_SCOPE, named(APPS_SCOPE)).also { appsScope ->
            getKoin().getScopeOrNull(SERVICE_CONNECTION_SCOPE)?.let { serviceConnectionScope ->
                appsScope.linkTo(serviceConnectionScope)
            }
        }
    private val viewModel by
        scope.viewModel<SplitTunnelingViewModel>(owner = { ViewModelOwner.from(this, this) })

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = viewModel.uiState.collectAsState().value
                    SplitTunnelingScreen(
                        uiState = state,
                        onShowSystemAppsClicked = viewModel::onShowSystemAppsClicked,
                        onExcludeAppClick = viewModel::onExcludeAppClick,
                        onIncludeAppClick = viewModel::onIncludeAppClick,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() }
                    )
                }
            }
        }
    }

    override fun onDestroy() {
        scope.close()
        super.onDestroy()
    }
}
