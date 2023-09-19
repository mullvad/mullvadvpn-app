package net.mullvad.mullvadvpn.ui.fragment

import android.app.Activity
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.AccountScreen
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel
import org.koin.core.parameter.parametersOf

class AccountFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {
    private val vm by viewModel<AccountViewModel>()

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    AccountScreen(
                        showSitePayment = IS_PLAY_BUILD.not(),
                        uiState = state,
                        viewActions = vm.viewActions,
                        enterTransitionEndAction = vm.enterTransitionEndAction,
                        onRedeemVoucherClick = { openRedeemVoucherFragment() },
                        onManageAccountClick = vm::onManageAccountClick,
                        onLogoutClick = vm::onLogoutClick,
                        onDeviceNameInfoClick = vm::onDeviceNameInfoClick,
                        onPurchaseBillingProductClick = vm::startBillingPayment,
                        onDialogClose = vm::closeDialog,
                        onTryVerificationAgain = vm::verifyPurchases,
                        onTryFetchProductsAgain = vm::fetchPaymentAvailability
                    ) {
                        activity?.onBackPressed()
                    }
                }
            }
        }
    }

    override fun onAttach(activity: Activity) {
        super.onAttach(activity)
        requireMainActivity().enterSecureScreen(this)
    }

    override fun onDetach() {
        super.onDetach()
        requireMainActivity().leaveSecureScreen(this)
    }

    override fun onEnterTransitionAnimationEnd() {
        vm.onTransitionAnimationEnd()
    }

    private fun openRedeemVoucherFragment() {
        val transaction = parentFragmentManager.beginTransaction()
        transaction.addToBackStack(null)
        RedeemVoucherDialogFragment().show(transaction, null)
    }
}
