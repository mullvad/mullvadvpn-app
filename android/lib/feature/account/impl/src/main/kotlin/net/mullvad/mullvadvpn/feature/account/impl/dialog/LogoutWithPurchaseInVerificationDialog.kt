package net.mullvad.mullvadvpn.feature.account.impl.dialog

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.account.api.LogoutPurchaseVerificationNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.NegativeConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Preview
@Composable
private fun PreviewLogoutWithPurchaseInVerificationDialog() {
    LogoutWithPurchaseInVerificationDialog(logout = {}, cancel = {})
}

@Composable
fun LogoutWithPurchaseInVerification(navigator: Navigator) {
    LogoutWithPurchaseInVerificationDialog(
        logout = { navigator.goBack(LogoutPurchaseVerificationNavResult) },
        cancel = { navigator.goBack() },
    )
}

@Composable
fun LogoutWithPurchaseInVerificationDialog(logout: () -> Unit, cancel: () -> Unit) {
    NegativeConfirmationDialog(
        message = stringResource(R.string.ongoing_verification_warning),
        messageStyle = MaterialTheme.typography.labelLarge,
        confirmationText = stringResource(R.string.log_out),
        cancelText = stringResource(R.string.cancel),
        onConfirm = logout,
        onBack = cancel,
    )
}
