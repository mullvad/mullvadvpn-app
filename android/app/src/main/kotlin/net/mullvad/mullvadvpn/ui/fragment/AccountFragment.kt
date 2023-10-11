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
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class AccountFragment : BaseFragment() {
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
                        uiState = state,
                        uiSideEffect = vm.uiSideEffect,
                        enterTransitionEndAction = vm.enterTransitionEndAction,
                        onRedeemVoucherClick = { openRedeemVoucherFragment() },
                        onManageAccountClick = vm::onManageAccountClick,
                        onLogoutClick = vm::onLogoutClick,
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() }
                    )
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
