package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.lib.theme.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewPaymentDialog() {
    PaymentDialog(
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

@Composable
fun PaymentDialog(
    title: String,
    message: String,
    icon: Int,
    onConfirmClick: () -> Unit,
    confirmText: String,
    dismissText: String? = null,
    onDismissClick: () -> Unit = {},
    onDismissRequest: () -> Unit
) {
    AlertDialog(
        title = {
            Column {
                Icon(
                    modifier = Modifier.fillMaxWidth().height(Dimens.iconHeight),
                    painter = painterResource(id = icon),
                    contentDescription = ""
                )
                Text(text = title, style = MaterialTheme.typography.headlineSmall)
            }
        },
        text = { Text(text = message, style = MaterialTheme.typography.bodySmall) },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        textContentColor = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaDescription),
        onDismissRequest = onDismissRequest,
        dismissButton = {
            dismissText?.let {
                ActionButton(
                    text = dismissText,
                    onClick = onDismissClick,
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.error,
                            contentColor = MaterialTheme.colorScheme.onError,
                        )
                )
            }
        },
        confirmButton = {
            ActionButton(
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                    ),
                text = confirmText,
                onClick = onConfirmClick
            )
        }
    )
}
