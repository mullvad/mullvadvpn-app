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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.textfield.GroupedTextField
import net.mullvad.mullvadvpn.compose.util.vouchersVisualTransformation
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.lib.theme.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import org.joda.time.DateTimeConstants

@Preview(device = Devices.TV_720p)
@Composable
private fun PreviewRedeemVoucherDialog() {
    AppTheme {
        RedeemVoucherDialog(
            uiState = VoucherDialogUiState.INITIAL,
            onVoucherInputChange = {},
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogVerifying() {
    AppTheme {
        RedeemVoucherDialog(
            uiState = VoucherDialogUiState("", VoucherDialogState.Verifying),
            onVoucherInputChange = {},
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, device = Devices.PIXEL_3)
@Composable
private fun PreviewRedeemVoucherDialogError() {
    AppTheme {
        RedeemVoucherDialog(
            uiState = VoucherDialogUiState("", VoucherDialogState.Error("An Error message")),
            onVoucherInputChange = {},
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
            uiState = VoucherDialogUiState("", VoucherDialogState.Success(3600)),
            onVoucherInputChange = {},
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Composable
fun RedeemVoucherDialog(
    uiState: VoucherDialogUiState,
    onVoucherInputChange: (String) -> Unit = {},
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        title = {
            if (uiState.voucherViewModelState !is VoucherDialogState.Success)
                Text(
                    text = stringResource(id = R.string.enter_voucher_code),
                    style = MaterialTheme.typography.titleMedium
                )
        },
        confirmButton = {
            Column {
                if (uiState.voucherViewModelState !is VoucherDialogState.Success) {
                    ActionButton(
                        text = stringResource(id = R.string.redeem),
                        onClick = { onRedeem(uiState.voucherInput) },
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
                        isEnabled = uiState.voucherInput.length == VOUCHER_LENGTH
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
                                if (uiState.voucherViewModelState is VoucherDialogState.Success)
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
                if (uiState.voucherViewModelState is VoucherDialogState.Success) {
                    val days: Int =
                        (uiState.voucherViewModelState.addedTime /
                                DateTimeConstants.SECONDS_PER_DAY)
                            .toInt()
                    val message =
                        stringResource(
                            R.string.added_to_your_account,
                            when (days) {
                                0 -> {
                                    stringResource(R.string.less_than_one_day)
                                }
                                in 1..59 -> {
                                    pluralStringResource(id = R.plurals.days, count = days, days)
                                }
                                else -> {
                                    pluralStringResource(
                                        id = R.plurals.months,
                                        count = days / 30,
                                        days / 30
                                    )
                                }
                            }
                        )
                    RedeemSuccessBody(message = message)
                } else {
                    EnterVoucherBody(
                        uiState = uiState,
                        onVoucherInputChange = onVoucherInputChange,
                        onRedeem = onRedeem
                    )
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss
    )
}

@Composable
private fun RedeemSuccessBody(message: String) {
    Image(
        painter = painterResource(R.drawable.icon_success),
        contentDescription = null, // No meaningful user info or action.
        modifier = Modifier.width(Dimens.buttonHeight).height(Dimens.buttonHeight)
    )

    Text(
        text = stringResource(id = R.string.voucher_success_title),
        modifier =
            Modifier.padding(start = Dimens.smallPadding, top = Dimens.screenVerticalMargin)
                .fillMaxWidth(),
        color = MaterialTheme.colorScheme.onPrimary,
        style = MaterialTheme.typography.titleMedium
    )

    Text(
        text = message,
        modifier =
            Modifier.padding(start = Dimens.smallPadding, top = Dimens.cellTopPadding)
                .fillMaxWidth(),
        style = MaterialTheme.typography.labelMedium
    )
}

@Composable
private fun EnterVoucherBody(
    uiState: VoucherDialogUiState,
    onVoucherInputChange: (String) -> Unit = {},
    onRedeem: (voucherCode: String) -> Unit
) {
    GroupedTextField(
        value = uiState.voucherInput,
        onSubmit = { input ->
            if (input.isNotEmpty()) {
                onRedeem(input)
            }
        },
        onValueChanged = { input -> onVoucherInputChange(input.uppercase()) },
        isValidValue = uiState.voucherInput.isNotEmpty(),
        keyboardType = KeyboardType.Text,
        placeholderText = stringResource(id = R.string.voucher_hint),
        placeHolderColor =
            MaterialTheme.colorScheme.onPrimary
                .copy(alpha = AlphaDisabled)
                .compositeOver(MaterialTheme.colorScheme.primary),
        visualTransformation = vouchersVisualTransformation(),
        maxCharLength = VOUCHER_LENGTH,
        onFocusChange = {},
        isDigitsOnlyAllowed = false,
        isEnabled = true,
        validateRegex = "^[A-Za-z0-9]*$".toRegex()
    )
    Spacer(modifier = Modifier.height(Dimens.smallPadding))
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.height(Dimens.listIconSize).fillMaxWidth()
    ) {
        if (uiState.voucherViewModelState is VoucherDialogState.Verifying) {
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
        } else if (uiState.voucherViewModelState is VoucherDialogState.Error) {
            Text(
                text = uiState.voucherViewModelState.errorMessage,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall
            )
        }
    }
}
