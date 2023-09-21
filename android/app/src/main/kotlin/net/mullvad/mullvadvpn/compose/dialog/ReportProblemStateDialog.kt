package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState

@Composable
fun ShowReportProblemStateDialog(
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
    ReportProblemSendingDialog()
}

@Composable
private fun ReportProblemSendingDialog() {
    AlertDialog(
        onDismissRequest = {},
        title = {
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier = Modifier.size(Dimens.progressIndicatorSize)
                )
            }
        },
        text = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text = stringResource(id = R.string.sending),
                    color = colorResource(id = R.color.white),
                    fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                    fontStyle = FontStyle.Normal,
                    textAlign = TextAlign.Start,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {},
        properties =
            DialogProperties(
                dismissOnClickOutside = false,
                dismissOnBackPress = false,
            ),
        containerColor = colorResource(id = R.color.darkBlue)
    )
}

@Preview
@Composable
private fun PreviewReportProblemSuccessDialog() {
    ReportProblemSuccessDialog(
        "Email@em.com",
        onConfirm = {},
    )
}

@Composable
fun ReportProblemSuccessDialog(email: String?, onConfirm: () -> Unit) {
    AlertDialog(
        onDismissRequest = { onConfirm() },
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.fillMaxWidth()
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_success),
                    contentDescription = "Remove",
                    modifier = Modifier.width(50.dp).height(50.dp)
                )
            }
        },
        text = {
            Text(
                text =
                    buildAnnotatedString {
                        withStyle(SpanStyle(color = colorResource(id = R.color.green))) {
                            append(stringResource(id = R.string.sent_thanks))
                        }
                        append(" ")

                        withStyle(SpanStyle(color = colorResource(id = R.color.white))) {
                            append(stringResource(id = R.string.we_will_look_into_this))
                        }
                    },
                fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                modifier = Modifier.fillMaxWidth()
            )
        },
        confirmButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.blue),
                        contentColor = Color.White
                    ),
                onClick = { onConfirm() },
            ) {
                Text(text = stringResource(id = R.string.dismiss), fontSize = 18.sp)
            }
        },
        containerColor = colorResource(id = R.color.darkBlue)
    )
}

@Preview
@Composable
private fun PreviewReportProblemErrorDialog() {
    ReportProblemErrorDialog(
        onDismiss = {},
        retry = {},
    )
}

@Composable
fun ReportProblemErrorDialog(onDismiss: () -> Unit, retry: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.fillMaxWidth()
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_fail),
                    contentDescription = null,
                    modifier = Modifier.width(50.dp).height(50.dp)
                )
            }
        },
        text = {
            Text(
                text = stringResource(id = R.string.failed_to_send_details),
                color = colorResource(id = R.color.white),
                fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                modifier = Modifier.fillMaxWidth()
            )
        },
        dismissButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.blue),
                        contentColor = Color.White
                    ),
                onClick = onDismiss,
            ) {
                Text(text = stringResource(id = R.string.edit_message), fontSize = 18.sp)
            }
        },
        confirmButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.green),
                        contentColor = Color.White
                    ),
                onClick = retry,
            ) {
                Text(text = stringResource(id = R.string.try_again), fontSize = 18.sp)
            }
        },
        containerColor = colorResource(id = R.color.darkBlue)
    )
}
