package net.mullvad.mullvadvpn.ui.fragment

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.ConnectScreen
import net.mullvad.mullvadvpn.lib.common.util.JobTracker
import net.mullvad.mullvadvpn.lib.common.util.appendHideNavOnReleaseBuild
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.paintNavigationBar
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.koin.androidx.viewmodel.ext.android.viewModel

class ConnectFragment : BaseFragment(), NavigationBarPainter {

    // Injected dependencies
    private val connectViewModel: ConnectViewModel by viewModel()

    private lateinit var headerBar: HeaderBar

    @Deprecated("Refactor code to instead rely on Lifecycle.") private val jobTracker = JobTracker()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.connect, container, false)

        headerBar =
            view.findViewById<HeaderBar>(R.id.header_bar).apply {
                tunnelState = connectViewModel.uiState.value.tunnelUiState
            }

        view.findViewById<ComposeView>(R.id.compose_view).setContent {
            AppTheme {
                val state = connectViewModel.uiState.collectAsState().value
                ConnectScreen(
                    uiState = state,
                    viewActions = connectViewModel.viewActions,
                    onDisconnectClick = connectViewModel::onDisconnectClick,
                    onReconnectClick = connectViewModel::onReconnectClick,
                    onConnectClick = connectViewModel::onConnectClick,
                    onCancelClick = connectViewModel::onCancelClick,
                    onSwitchLocationClick = { openSwitchLocationScreen() },
                    onToggleTunnelInfo = connectViewModel::toggleTunnelInfoExpansion,
                    onUpdateVersionClick = { openDownloadUrl() },
                    onShowAccountClick = connectViewModel::onShowAccountClick
                )
            }
        }

        return view
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.blue))
    }

    private fun openDownloadUrl() {
        val intent =
            Intent(
                    Intent.ACTION_VIEW,
                    Uri.parse(
                        requireContext()
                            .getString(R.string.download_url)
                            .appendHideNavOnReleaseBuild()
                    )
                )
                .apply { flags = Intent.FLAG_ACTIVITY_NEW_TASK }
        requireContext().startActivity(intent)
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) { launchViewModelSubscription() }
    }

    private fun CoroutineScope.launchViewModelSubscription() = launch {
        connectViewModel.uiState.collect { uiState -> updateTunnelState(uiState.tunnelRealState) }
    }

    private fun updateTunnelState(realState: TunnelState) {
        headerBar.tunnelState = realState

        if (realState.isTunnelErrorStateDueToExpiredAccount()) {
            openOutOfTimeScreen()
        }
    }

    private fun openSwitchLocationScreen() {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SelectLocationFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    private fun openOutOfTimeScreen() {
        jobTracker.newUiJob("openOutOfTimeScreen") {
            parentFragmentManager.beginTransaction().apply {
                replace(R.id.main_fragment, OutOfTimeFragment())
                commitAllowingStateLoss()
            }
        }
    }

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount()
            ?: false
    }
}
