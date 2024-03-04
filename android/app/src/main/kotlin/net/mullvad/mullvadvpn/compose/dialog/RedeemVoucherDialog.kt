package net.mullvad.mullvadvpn.compose.dialog

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.SecureFlagPolicy
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.test.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.compose.util.MAX_VOUCHER_LENGTH
import net.mullvad.mullvadvpn.compose.util.vouchersVisualTransformation
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import org.joda.time.DateTimeConstants
import org.koin.androidx.compose.koinViewModel

@Preview(device = Devices.TV_720p)
@Composable
private fun PreviewRedeemVoucherDialog() {
    AppTheme {
        RedeemVoucherDialog(
            state = VoucherDialogUiState.INITIAL,
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
            state = VoucherDialogUiState("", VoucherDialogState.Verifying),
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
            state = VoucherDialogUiState("", VoucherDialogState.Error("An Error message")),
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
            state = VoucherDialogUiState("", VoucherDialogState.Success(3600)),
            onVoucherInputChange = {},
            onRedeem = {},
            onDismiss = {}
        )
    }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun RedeemVoucher(resultBackNavigator: ResultBackNavigator<Boolean>) {
    val vm = koinViewModel<VoucherDialogViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    RedeemVoucherDialog(
        state = state,
        onVoucherInputChange = vm::onVoucherInputChange,
        onRedeem = vm::onRedeem,
        onDismiss = { resultBackNavigator.navigateBack(result = it) }
    )
}

@Composable
fun RedeemVoucherDialog(
    state: VoucherDialogUiState,
    onVoucherInputChange: (String) -> Unit = {},
    onRedeem: (voucherCode: String) -> Unit,
    onDismiss: (isTimeAdded: Boolean) -> Unit
) {
    AlertDialog(
        title = {
            if (state.voucherState !is VoucherDialogState.Success)
                Text(
                    text = stringResource(id = R.string.enter_voucher_code),
                )
        },
        confirmButton = {
            Column {
                if (state.voucherState !is VoucherDialogState.Success) {
                    VariantButton(
                        text = stringResource(id = R.string.redeem),
                        onClick = { onRedeem(state.voucherInput) },
                        modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                        isEnabled = state.voucherInput.length == VOUCHER_LENGTH
                    )
                }
                PrimaryButton(
                    text =
                        stringResource(
                            id =
                                if (state.voucherState is VoucherDialogState.Success)
                                    R.string.got_it
                                else R.string.cancel
                        ),
                    onClick = { onDismiss(state.voucherState is VoucherDialogState.Success) }
                )
            }
        },
        text = {
            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                if (state.voucherState is VoucherDialogState.Success) {
                    val days: Int =
                        (state.voucherState.addedTime / DateTimeConstants.SECONDS_PER_DAY).toInt()
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
                        state = state,
                        onVoucherInputChange = onVoucherInputChange,
                        onRedeem = onRedeem
                    )
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = { onDismiss(state.voucherState is VoucherDialogState.Success) },
        properties =
            DialogProperties(
                securePolicy =
                    if (BuildConfig.DEBUG) SecureFlagPolicy.Inherit else SecureFlagPolicy.SecureOn
            )
    )
}

@Composable
private fun RedeemSuccessBody(message: String) {
    Image(
        painter = painterResource(R.drawable.icon_success),
        contentDescription = null,
        modifier = Modifier.fillMaxWidth().height(Dimens.buttonHeight)
    )
    Text(
        text = stringResource(id = R.string.voucher_success_title),
        modifier =
            Modifier.padding(
                    start = Dimens.smallPadding,
                    top = Dimens.successIconVerticalPadding,
                )
                .fillMaxWidth(),
        color = MaterialTheme.colorScheme.onPrimary,
        style = MaterialTheme.typography.titleMedium
    )

    Text(
        text = message,
        modifier =
            Modifier.padding(start = Dimens.smallPadding, top = Dimens.cellTopPadding)
                .fillMaxWidth(),
        color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDescription),
        style = MaterialTheme.typography.labelMedium
    )
}

@Composable
private fun EnterVoucherBody(
    state: VoucherDialogUiState,
    onVoucherInputChange: (String) -> Unit = {},
    onRedeem: (voucherCode: String) -> Unit
) {
    CustomTextField(
        value = state.voucherInput,
        onSubmit = { input ->
            if (state.voucherInput.length == VOUCHER_LENGTH) {
                onRedeem(input)
            }
        },
        onValueChanged = { input -> onVoucherInputChange(input) },
        isValidValue =
            state.voucherInput.isEmpty() || state.voucherInput.length == MAX_VOUCHER_LENGTH,
        keyboardType = KeyboardType.Password,
        placeholderText = stringResource(id = R.string.voucher_hint),
        visualTransformation = vouchersVisualTransformation(),
        isDigitsOnlyAllowed = false,
        modifier = Modifier.testTag(VOUCHER_INPUT_TEST_TAG)
    )
    Spacer(modifier = Modifier.height(Dimens.smallPadding))
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.height(Dimens.listIconSize).fillMaxWidth()
    ) {
        if (state.voucherState is VoucherDialogState.Verifying) {
            MullvadCircularProgressIndicatorSmall()
            Text(
                text = stringResource(id = R.string.verifying_voucher),
                modifier = Modifier.padding(start = Dimens.smallPadding),
                color = MaterialTheme.colorScheme.onPrimary,
                style = MaterialTheme.typography.bodySmall
            )
        } else if (state.voucherState is VoucherDialogState.Error) {
            Text(
                text = state.voucherState.errorMessage,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall
            )
        }
    }
}
