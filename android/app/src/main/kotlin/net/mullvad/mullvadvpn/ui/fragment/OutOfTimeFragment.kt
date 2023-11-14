package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.OutOfTimeScreen
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class OutOfTimeFragment : BaseFragment() {

    private val vm by viewModel<OutOfTimeViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    OutOfTimeScreen(
                        showSitePayment = IS_PLAY_BUILD.not(),
                        uiState = state,
                        uiSideEffect = vm.uiSideEffect,
                        onSitePaymentClick = vm::onSitePaymentClick,
                        onRedeemVoucherClick = ::openRedeemVoucherFragment,
                        onSettingsClick = ::openSettingsView,
                        onAccountClick = ::openAccountView,
                        openConnectScreen = ::advanceToConnectScreen,
                        onDisconnectClick = vm::onDisconnectClick,
                        onPurchaseBillingProductClick = vm::startBillingPayment,
                        onRetryFetchProducts = vm::onRetryFetchProducts,
                        onRetryVerification = vm::onRetryVerification,
                        onClosePurchaseResultDialog = vm::onClosePurchaseResultDialog,
                    )
                }
            }
        }
    }

    private fun openRedeemVoucherFragment() {
        val transaction = parentFragmentManager.beginTransaction()
        transaction.addToBackStack(null)
        RedeemVoucherDialogFragment { wasSuccessful -> if (wasSuccessful) advanceToConnectScreen() }
            .show(transaction, null)
    }

    private fun advanceToConnectScreen() {
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, ConnectFragment())
            commitAllowingStateLoss()
        }
    }

    private fun openSettingsView() {
        (context as? MainActivity)?.openSettings()
    }

    private fun openAccountView() {
        (context as? MainActivity)?.openAccount()
    }
}
