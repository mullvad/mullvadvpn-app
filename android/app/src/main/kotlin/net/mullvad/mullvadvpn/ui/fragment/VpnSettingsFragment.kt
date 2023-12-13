package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.VpnSettingsScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class VpnSettingsFragment : BaseFragment() {
    private val vm by viewModel<VpnSettingsViewModel>()

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
                        onSaveMtuClick = vm::onSaveMtuClick,
                        onAutoConnectClick = { openAutoConnectAndLockdownModeFragment() },
                        onRestoreMtuClick = vm::onRestoreMtuClick,
                        onCancelMtuDialogClick = vm::onCancelDialogClick,
                        onToggleAutoConnect = vm::onToggleAutoConnect,
                        onToggleLocalNetworkSharing = vm::onToggleLocalNetworkSharing,
                        onToggleDnsClick = vm::onToggleDnsClick,
                        onToggleBlockAds = vm::onToggleBlockAds,
                        onToggleBlockTrackers = vm::onToggleBlockTrackers,
                        onToggleBlockMalware = vm::onToggleBlockMalware,
                        onToggleBlockAdultContent = vm::onToggleBlockAdultContent,
                        onToggleBlockGambling = vm::onToggleBlockGambling,
                        onToggleBlockSocialMedia = vm::onToggleBlockSocialMedia,
                        onDnsClick = vm::onDnsClick,
                        onDnsInputChange = vm::onDnsInputChange,
                        onSaveDnsClick = vm::onSaveDnsClick,
                        onRemoveDnsClick = vm::onRemoveDnsClick,
                        onCancelDnsDialogClick = vm::onCancelDns,
                        onLocalNetworkSharingInfoClick = vm::onLocalNetworkSharingInfoClick,
                        onContentsBlockersInfoClick = vm::onContentsBlockerInfoClick,
                        onCustomDnsInfoClick = vm::onCustomDnsInfoClick,
                        onMalwareInfoClick = vm::onMalwareInfoClick,
                        onDismissInfoClick = vm::onDismissInfoClick,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() },
                        onStopEvent = vm::onStopEvent,
                        toastMessagesSharedFlow = vm.toastMessages,
                        onSelectObfuscationSetting = vm::onSelectObfuscationSetting,
                        onObfuscationInfoClick = vm::onObfuscationInfoClick,
                        onSelectQuantumResistanceSetting = vm::onSelectQuantumResistanceSetting,
                        onQuantumResistanceInfoClicked = vm::onQuantumResistanceInfoClicked,
                        onWireguardPortSelected = vm::onWireguardPortSelected,
                        onWireguardPortInfoClicked = vm::onWireguardPortInfoClicked,
                        onShowCustomPortDialog = vm::onShowCustomPortDialog,
                        onCancelCustomPortDialogClick = vm::onCancelDialogClick,
                        onCloseCustomPortDialog = vm::onCancelDialogClick
                    )
                }
            }
        }
    }

    private fun openFragment(fragment: Fragment) {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, fragment)
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    private fun openAutoConnectAndLockdownModeFragment() {
        openFragment(AutoConnectAndLockdownModeFragment())
    }
}
