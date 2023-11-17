package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewReportProblemNoEmailDialog() {
    AppTheme {
        ReportProblemNoEmailDialog(
            onDismiss = {},
            onConfirm = {},
        )
    }
}

@Composable
fun ReportProblemNoEmailDialog(onDismiss: () -> Unit, onConfirm: () -> Unit) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
        icon = {
            Icon(
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = null,
                modifier = Modifier.size(Dimens.dialogIconHeight),
                tint = Color.Unspecified
            )
        },
        text = {
            Text(
                text = stringResource(id = R.string.confirm_no_email),
                modifier = Modifier.fillMaxWidth(),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onBackground
            )
        },
        dismissButton = {
            NegativeButton(
                modifier = Modifier.fillMaxWidth(),
                onClick = onConfirm,
                text = stringResource(id = R.string.send_anyway)
            )
        },
        confirmButton = {
            PrimaryButton(
                modifier = Modifier.fillMaxWidth(),
                onClick = { onDismiss() },
                text = stringResource(id = R.string.back)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
