package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.WelcomeScreen
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class WelcomeFragment : BaseFragment() {

    private val vm by viewModel<WelcomeViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    WelcomeScreen(
                        showSitePayment = IS_PLAY_BUILD.not(),
                        uiState = state,
                        viewActions = vm.viewActions,
                        onSitePaymentClick = vm::onSitePaymentClick,
                        onRedeemVoucherClick = ::openRedeemVoucherFragment,
                        onSettingsClick = ::openSettingsView,
                        onAccountClick = ::openAccountView,
                        openConnectScreen = ::advanceToConnectScreen,
                        onPurchaseBillingProductClick = vm::startBillingPayment,
                        onDialogClose = vm::closeDialog,
                        onTryVerificationAgain = vm::verifyPurchases,
                        onTryFetchProductsAgain = vm::fetchPaymentAvailability
                    )
                }
            }
        }
    }

    private fun openRedeemVoucherFragment() {
        val transaction = parentFragmentManager.beginTransaction()
        transaction.addToBackStack(null)
        RedeemVoucherDialogFragment().show(transaction, null)
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
