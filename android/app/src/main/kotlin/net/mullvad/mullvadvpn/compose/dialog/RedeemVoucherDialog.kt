package net.mullvad.mullvadvpn.compose.dialog

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.textfield.GroupedTextField
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.lib.theme.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview(device = Devices.TV_720p)
@Composable
private fun PreviewRedeemVoucherDialog() {
    AppTheme {
        RedeemVoucherDialog(uiState = VoucherDialogUiState.Default, onRedeem = {}, onDismiss = {})
    }
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogVerifying() {
    AppTheme {
        RedeemVoucherDialog(uiState = VoucherDialogUiState.Verifying, onRedeem = {}, onDismiss = {})
    }
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogError() {
    AppTheme {
        RedeemVoucherDialog(
            uiState = VoucherDialogUiState.Error("An Error message"),
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogSuccess() {
    AppTheme {
        RedeemVoucherDialog(
            uiState = VoucherDialogUiState.Success("success message"),
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Composable
fun RedeemVoucherDialog(
    uiState: VoucherDialogUiState,
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: () -> Unit
) {
    val voucher = remember { mutableStateOf("") }

    AlertDialog(
        title = {
            if (uiState !is VoucherDialogUiState.Success)
                Text(
                    text = stringResource(id = R.string.enter_voucher_code),
                    style = MaterialTheme.typography.titleMedium
                )
        },
        confirmButton = {
            Column {
                if (uiState !is VoucherDialogUiState.Success) {
                    ActionButton(
                        text = stringResource(id = R.string.redeem),
                        onClick = { onRedeem(voucher.value) },
                        modifier = Modifier.padding(bottom = Dimens.mediumPadding),
                        colors =
                            ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.surface,
                                contentColor = MaterialTheme.colorScheme.onSurface,
                                disabledContentColor =
                                    MaterialTheme.colorScheme.onSurface
                                        .copy(alpha = AlphaInactive)
                                        .compositeOver(MaterialTheme.colorScheme.surface),
                                disabledContainerColor =
                                    MaterialTheme.colorScheme.surface
                                        .copy(alpha = AlphaDisabled)
                                        .compositeOver(MaterialTheme.colorScheme.surface)
                            ),
                        isEnabled = voucher.value.length == VOUCHER_LENGTH
                    )
                }
                ActionButton(
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                    text =
                        stringResource(
                            id =
                                if (uiState is VoucherDialogUiState.Success)
                                    R.string.changes_dialog_dismiss_button
                                else R.string.cancel
                        ),
                    onClick = onDismiss
                )
            }
        },
        text = {
            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                if (uiState is VoucherDialogUiState.Success) {
                    Image(
                        painter = painterResource(R.drawable.icon_success),
                        contentDescription = null, // No meaningful user info or action.
                        modifier = Modifier.width(Dimens.buttonHeight).height(Dimens.buttonHeight)
                    )

                    Text(
                        text = stringResource(id = R.string.voucher_success_title),
                        modifier =
                            Modifier.padding(
                                    start = Dimens.smallPadding,
                                    top = Dimens.screenVerticalMargin
                                )
                                .fillMaxWidth(),
                        color = MaterialTheme.colorScheme.onPrimary,
                        style = MaterialTheme.typography.titleMedium
                    )

                    Text(
                        text = uiState.message,
                        modifier =
                            Modifier.padding(
                                    start = Dimens.smallPadding,
                                    top = Dimens.cellTopPadding
                                )
                                .fillMaxWidth(),
                        style = MaterialTheme.typography.labelMedium
                    )
                } else {
                    GroupedTextField(
                        value = voucher.value,
                        onSubmit = { input ->
                            if (input.isNotEmpty()) {
                                onRedeem(input)
                            }
                        },
                        onValueChanged = { input -> voucher.value = input.uppercase() },
                        isValidValue = voucher.value.isNotEmpty(),
                        keyboardType = KeyboardType.Text,
                        placeholderText = stringResource(id = R.string.voucher_hint),
                        placeHolderColor =
                            MaterialTheme.colorScheme.onPrimary
                                .copy(alpha = AlphaDisabled)
                                .compositeOver(MaterialTheme.colorScheme.primary),
                        visualTransformation = { voucher ->
                            vouchersVisualTransformation(voucher, VOUCHER_LENGTH)
                        },
                        maxCharLength = VOUCHER_LENGTH,
                        onFocusChange = {},
                        isDigitsOnlyAllowed = false,
                        isEnabled = true,
                        validateRegex = "^[A-Za-z0-9 -]*$".toRegex()
                    )
                    Spacer(modifier = Modifier.height(Dimens.smallPadding))
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.height(Dimens.listIconSize).fillMaxWidth()
                    ) {
                        if (uiState is VoucherDialogUiState.Verifying) {
                            CircularProgressIndicator(
                                modifier =
                                    Modifier.height(Dimens.loadingSpinnerSizeMedium)
                                        .width(Dimens.loadingSpinnerSizeMedium),
                                color = MaterialTheme.colorScheme.onSecondary
                            )
                            Text(
                                text = stringResource(id = R.string.verifying_voucher),
                                modifier = Modifier.padding(start = Dimens.smallPadding),
                                color = MaterialTheme.colorScheme.onPrimary,
                                style = MaterialTheme.typography.bodySmall
                            )
                        } else if (uiState is VoucherDialogUiState.Error) {
                            Text(
                                text = uiState.errorMessage,
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }
                    }
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss
    )
}

const val ACCOUNT_TOKEN_CHUNK_SIZE = 4

private fun vouchersVisualTransformation(text: AnnotatedString, maxSize: Int): TransformedText {
    val trimmed =
        if (text.text.length >= maxSize) text.text.substring(0 until maxSize) else text.text
    var out = ""
    var transformedMaxSize = maxSize + maxSize / ACCOUNT_TOKEN_CHUNK_SIZE

    for (i in trimmed.indices) {
        out += trimmed[i]
        if (i % 4 == 3 && i != 15) out += "-"
    }
    val voucherOffsetTranslator =
        object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int =
                (offset + offset / ACCOUNT_TOKEN_CHUNK_SIZE).coerceAtMost(transformedMaxSize - 1)

            override fun transformedToOriginal(offset: Int): Int =
                offset - offset / ACCOUNT_TOKEN_CHUNK_SIZE
        }

    return TransformedText(AnnotatedString(out), voucherOffsetTranslator)
}
