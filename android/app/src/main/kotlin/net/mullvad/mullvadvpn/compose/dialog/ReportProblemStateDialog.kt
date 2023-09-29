package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState

@Composable
fun ShowReportProblemState(
    sendingState: SendingReportUiState,
    onDismiss: () -> Unit,
    onClearForm: () -> Unit,
    retry: () -> Unit
) {
    when (sendingState) {
        SendingReportUiState.Sending -> ReportProblemSendingDialog()
        is SendingReportUiState.Error ->
            ReportProblemErrorDialog(onDismiss = onDismiss, retry = retry)
        is SendingReportUiState.Success ->
            ReportProblemSuccessDialog(
                sendingState.email,
                onConfirm = {
                    onClearForm()
                    onDismiss()
                }
            )
    }
}

@Preview
@Composable
private fun PreviewReportProblemSendingDialog() {
    AppTheme { ReportProblemSendingDialog() }
}

@Composable
private fun ReportProblemSendingDialog() {
    AlertDialog(
        onDismissRequest = {},
        icon = {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.size(Dimens.progressIndicatorSize)
            )
        },
        title = { Text(text = stringResource(id = R.string.sending)) },
        confirmButton = {},
        properties =
            DialogProperties(
                dismissOnClickOutside = false,
                dismissOnBackPress = false,
            ),
        containerColor = MaterialTheme.colorScheme.background
    )
}

@Preview
@Composable
private fun PreviewReportProblemSuccessDialog() {
    AppTheme {
        ReportProblemSuccessDialog(
            "Email@em.com",
            onConfirm = {},
        )
    }
}

@Composable
fun ReportProblemSuccessDialog(email: String?, onConfirm: () -> Unit) {
    AlertDialog(
        onDismissRequest = { onConfirm() },
        icon = {
            Icon(
                painter = painterResource(id = R.drawable.icon_success),
                contentDescription = stringResource(id = R.string.sent),
                modifier = Modifier.size(Dimens.dialogIconHeight),
                tint = Color.Unspecified
            )
        },
        title = { Text(text = stringResource(id = R.string.sent)) },
        text = {
            Column {
                Text(
                    text =
                        buildAnnotatedString {
                            withStyle(SpanStyle(color = MaterialTheme.colorScheme.surface)) {
                                append(stringResource(id = R.string.sent_thanks))
                            }
                            append(" ")
                            withStyle(SpanStyle(color = MaterialTheme.colorScheme.onPrimary)) {
                                append(stringResource(id = R.string.we_will_look_into_this))
                            }
                        },
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.fillMaxWidth()
                )

                Spacer(modifier = Modifier.height(Dimens.smallPadding))
                email?.let {
                    val emailTemplate = stringResource(R.string.sent_contact)
                    val annotatedEmailString =
                        remember(it) {
                            val emailStart = emailTemplate.indexOf('%')

                            buildAnnotatedString {
                                append(emailTemplate.substring(0, emailStart))
                                withStyle(SpanStyle(fontWeight = FontWeight.Bold)) { append(email) }
                            }
                        }

                    Text(
                        text = annotatedEmailString,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        },
        confirmButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                    ),
                onClick = { onConfirm() },
                text = stringResource(id = R.string.dismiss)
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
    )
}

@Preview
@Composable
private fun PreviewReportProblemErrorDialog() {
    AppTheme {
        ReportProblemErrorDialog(
            onDismiss = {},
            retry = {},
        )
    }
}

@Composable
fun ReportProblemErrorDialog(onDismiss: () -> Unit, retry: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = stringResource(id = R.string.failed_to_send),
                modifier = Modifier.size(Dimens.dialogIconHeight),
                tint = Color.Unspecified
            )
        },
        title = { Text(text = stringResource(id = R.string.failed_to_send)) },
        text = {
            Text(
                text = stringResource(id = R.string.failed_to_send_details),
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.fillMaxWidth()
            )
        },
        dismissButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                    ),
                onClick = onDismiss,
                text = stringResource(id = R.string.edit_message)
            )
        },
        confirmButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.surface,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                    ),
                onClick = retry,
                text = stringResource(id = R.string.try_again)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
