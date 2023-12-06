package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.compose.screen.SplitTunnelingScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.android.ext.android.inject
import org.koin.androidx.viewmodel.ext.android.viewModel

class SplitTunnelingFragment : BaseFragment() {
    private val viewModel: SplitTunnelingViewModel by viewModel()
    private val applicationsIconManager: ApplicationsIconManager by inject()

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
                        onShowSplitTunneling = viewModel::enableSplitTunneling,
                        onShowSystemAppsClick = viewModel::onShowSystemAppsClick,
                        onExcludeAppClick = viewModel::onExcludeAppClick,
                        onIncludeAppClick = viewModel::onIncludeAppClick,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() },
                        onResolveIcon = { packageName ->
                            applicationsIconManager.getAppIcon(packageName)
                        }
                    )
                }
            }
        }
    }
}
