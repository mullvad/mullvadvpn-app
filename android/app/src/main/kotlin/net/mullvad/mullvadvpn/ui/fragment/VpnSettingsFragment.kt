package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.VpnSettingsScreen
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class VpnSettingsFragment : BaseFragment() {
    private val vm by viewModel<VpnSettingsViewModel>()

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
                    VpnSettingsScreen(
                        uiState = state,
                        onMtuCellClick = vm::onMtuCellClick,
                        onMtuInputChange = vm::onMtuInputChange,
                        onSaveMtuClick = vm::onSaveMtuClick,
                        onRestoreMtuClick = vm::onRestoreMtuClick,
                        onCancelMtuDialogClicked = vm::onCancelDialogClick,
                        onSplitTunnelingNavigationClick = ::openSplitTunnelingFragment,
                        onToggleAutoConnect = vm::onToggleAutoConnect,
                        onToggleLocalNetworkSharing = vm::onToggleLocalNetworkSharing,
                        onToggleDnsClick = vm::onToggleDnsClick,
                        onToggleBlockAds = vm::onToggleBlockAds,
                        onToggleBlockTrackers = vm::onToggleBlockTrackers,
                        onToggleBlockMalware = vm::onToggleBlockMalware,
                        onToggleBlockAdultContent = vm::onToggleBlockAdultContent,
                        onToggleBlockGambling = vm::onToggleBlockGambling,
                        onDnsClick = vm::onDnsClick,
                        onDnsInputChange = vm::onDnsInputChange,
                        onSaveDnsClick = vm::onSaveDnsClick,
                        onRemoveDnsClick = vm::onRemoveDnsClick,
                        onCancelDnsDialogClick = vm::onCancelDialogClick,
                        onLocalNetworkSharingInfoClick = vm::onLocalNetworkSharingInfoClick,
                        onContentsBlockersInfoClicked = vm::onContentsBlockerInfoClick,
                        onCustomDnsInfoClicked = vm::onCustomDnsInfoClick,
                        onMalwareInfoClicked = vm::onMalwareInfoClick,
                        onDismissInfoClicked = vm::onDismissInfoClick,
                        onBackClick = { activity?.onBackPressed() },
                        onStopEvent = vm::onStopEvent,
                        toastMessagesSharedFlow = vm.toastMessages,
                        onSelectObfuscationSetting = vm::onSelectObfuscationSetting,
                        onObfuscationInfoClicked = vm::onObfuscationInfoClicked
                    )
                }
            }
        }
    }

    private fun openSplitTunnelingFragment() {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, SplitTunnelingFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }
}
