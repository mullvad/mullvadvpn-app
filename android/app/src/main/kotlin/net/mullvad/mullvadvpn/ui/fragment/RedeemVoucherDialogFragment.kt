package net.mullvad.mullvadvpn.ui.fragment

import android.app.Dialog
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.DialogFragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.RedeemVoucherDialogScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class RedeemVoucherDialogFragment(val onDialogDismiss: (Boolean) -> Unit = {}) : DialogFragment() {

    private val vm by viewModel<VoucherDialogViewModel>()
    private lateinit var voucherDialog: Dialog

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    RedeemVoucherDialogScreen(
                        uiState = vm.uiState.collectAsState().value,
                        onVoucherInputChange = { vm.onVoucherInputChange(it) },
                        onRedeem = { vm.onRedeem(it) },
                        onDismiss = {
                            onDismiss(voucherDialog)
                            onDialogDismiss(it)
                        },
                        voucherValidator = { vm.validateVoucher(it) }
                    )
                }
            }
        }
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        voucherDialog = super.onCreateDialog(savedInstanceState)
        return voucherDialog
    }
}
