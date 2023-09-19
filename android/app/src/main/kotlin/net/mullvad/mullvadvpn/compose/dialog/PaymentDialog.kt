package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription

@Preview
@Composable
private fun PreviewBasePaymentDialog() {
    AppTheme {
        BasePaymentDialog(
            title = "Payment was unsuccessful",
            message = "We were unable to verify your paymenmt, please try again",
            icon = R.drawable.icon_fail,
            onConfirmClick = {},
            confirmText = "Cancel",
            onDismissRequest = {},
            dismissText = "Try again",
            onDismissClick = {},
        )
    }
}

@Composable
internal fun BasePaymentDialog(
    title: String,
    message: String,
    icon: Int,
    onConfirmClick: () -> Unit,
    confirmText: String,
    dismissText: String? = null,
    onDismissClick: (() -> Unit)? = null,
    onDismissRequest: () -> Unit
) {
    AlertDialog(
        icon = {
            Image(
                modifier = Modifier.fillMaxWidth().height(Dimens.iconHeight),
                painter = painterResource(id = icon),
                contentDescription = ""
            )
        },
        title = { Text(text = title, style = MaterialTheme.typography.headlineSmall) },
        text = { Text(text = message, style = MaterialTheme.typography.bodySmall) },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        iconContentColor = Color.Unspecified,
        textContentColor =
            MaterialTheme.colorScheme.onBackground
                .copy(alpha = AlphaDescription)
                .compositeOver(MaterialTheme.colorScheme.background),
        onDismissRequest = onDismissRequest,
        dismissButton = {
            dismissText?.let { NegativeButton(text = dismissText, onClick = onDismissClick ?: {}) }
        },
        confirmButton = { PrimaryButton(text = confirmText, onClick = onConfirmClick) }
    )
}
