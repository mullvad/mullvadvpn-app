package net.mullvad.mullvadvpn.compose.dialog

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
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
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialog() {
    AppTheme {
        RedeemVoucherDialog(uiState = VoucherDialogUiState(null), onRedeem = {}, onDismiss = {})
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
            Text(
                text = stringResource(id = R.string.enter_voucher_code),
                style = MaterialTheme.typography.titleMedium
            )
        },
        confirmButton = {
            Column {
                ActionButton(
                    text = stringResource(id = R.string.redeem),
                    onClick = { onRedeem(voucher.value) },
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.surface,
                            contentColor = MaterialTheme.colorScheme.onSurface,
                            disabledContentColor =
                                MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaInactive),
                            disabledContainerColor =
                                MaterialTheme.colorScheme.surface.copy(alpha = AlphaDisabled)
                        ),
                    isEnabled = voucher.value.length == 16
                )
                ActionButton(
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                    text = stringResource(id = R.string.cancel),
                    modifier = Modifier.padding(top = Dimens.mediumPadding),
                    onClick = onDismiss
                )
            }
        },
        text = {
            Column {
                GroupedTextField(
                    value = voucher.value,
                    onSubmit = { input ->
                        if (input.isNotEmpty()) {
                            onRedeem(input)
                        }
                    },
                    onValueChanged = { input ->
                        voucher.value = input.uppercase().format().replace(" ", "")
                    },
                    isValidValue = voucher.value.isNotEmpty(),
                    keyboardType = KeyboardType.Text,
                    placeholderText = stringResource(id = R.string.voucher_hint),
                    placeHolderColor =
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
                    visualTransformation = { voucher -> formatOtherVouchers(voucher) },
                    maxCharLength = VOUCHER_LENGTH,
                    onFocusChange = {},
                    isDigitsOnlyAllowed = false,
                    isEnabled = true,
                    validateRegex = "^[A-Za-z0-9 ]*$".toRegex()
                )
                Spacer(modifier = Modifier.height(Dimens.smallPadding))
                if (uiState.error != null) {
                    Text(
                        text = uiState.error,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss
    )
}

private fun formatOtherVouchers(text: AnnotatedString): TransformedText {
    val trimmed = if (text.text.length >= 16) text.text.substring(0..15) else text.text
    var out = ""

    for (i in trimmed.indices) {
        out += trimmed[i]
        if (i % 4 == 3 && i != 15) out += "-"
    }
    val voucherOffsetTranslator =
        object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                if (offset <= 3) return offset
                if (offset <= 7) return offset + 1
                if (offset <= 11) return offset + 2
                if (offset <= 16) return offset + 3
                return 19
            }

            override fun transformedToOriginal(offset: Int): Int {
                if (offset <= 4) return offset
                if (offset <= 9) return offset - 1
                if (offset <= 14) return offset - 2
                if (offset <= 19) return offset - 3
                return 16
            }
        }

    return TransformedText(AnnotatedString(out), voucherOffsetTranslator)
}
