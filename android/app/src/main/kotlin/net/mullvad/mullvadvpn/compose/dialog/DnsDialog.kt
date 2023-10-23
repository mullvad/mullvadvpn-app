package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.DnsTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.MullvadRed
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns

@Preview
@Composable
private fun PreviewDnsDialogNew() {
    AppTheme {
        DnsDialog(
            stagedDns =
                StagedDns.NewDns(CustomDnsItem.default(), StagedDns.ValidationResult.Success),
            isAllowLanEnabled = true,
            onIpAddressChanged = {},
            onAttemptToSave = {},
            onRemove = {},
            onDismiss = {}
        )
    }
}

@Preview
@Composable
private fun PreviewDnsDialogEdit() {
    AppTheme {
        DnsDialog(
            stagedDns =
                StagedDns.EditDns(
                    CustomDnsItem("1.1.1.1", false),
                    StagedDns.ValidationResult.Success,
                    0
                ),
            isAllowLanEnabled = true,
            onIpAddressChanged = {},
            onAttemptToSave = {},
            onRemove = {},
            onDismiss = {}
        )
    }
}

@Preview
@Composable
private fun PreviewDnsDialogEditAllowLanDisabled() {
    AppTheme {
        DnsDialog(
            stagedDns =
                StagedDns.EditDns(
                    CustomDnsItem(address = "1.1.1.1", isLocal = true),
                    StagedDns.ValidationResult.Success,
                    0
                ),
            isAllowLanEnabled = false,
            onIpAddressChanged = {},
            onAttemptToSave = {},
            onRemove = {},
            onDismiss = {}
        )
    }
}

@Composable
fun DnsDialog(
    stagedDns: StagedDns,
    isAllowLanEnabled: Boolean,
    onIpAddressChanged: (String) -> Unit,
    onAttemptToSave: () -> Unit,
    onRemove: () -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        title = {
            Text(
                text =
                    if (stagedDns is StagedDns.NewDns) {
                        stringResource(R.string.add_dns_server_dialog_title)
                    } else {
                        stringResource(R.string.update_dns_server_dialog_title)
                    },
                color = Color.White,
                style = MaterialTheme.typography.headlineSmall.copy(fontWeight = FontWeight.Normal)
            )
        },
        text = {
            Column {
                DnsTextField(
                    value = stagedDns.item.address,
                    isValidValue = stagedDns.isValid(),
                    onValueChanged = { newMtuValue -> onIpAddressChanged(newMtuValue) },
                    onSubmit = { onAttemptToSave() },
                    isEnabled = true,
                    placeholderText = stringResource(R.string.custom_dns_hint),
                    modifier = Modifier.fillMaxWidth()
                )

                val errorMessage =
                    when {
                        stagedDns.validationResult is
                            StagedDns.ValidationResult.DuplicateAddress -> {
                            stringResource(R.string.duplicate_address_warning)
                        }
                        stagedDns.item.isLocal && isAllowLanEnabled.not() -> {
                            stringResource(id = R.string.confirm_local_dns)
                        }
                        else -> {
                            null
                        }
                    }

                if (errorMessage != null) {
                    Text(
                        text = errorMessage,
                        style = MaterialTheme.typography.bodySmall,
                        color = MullvadRed,
                        modifier = Modifier.padding(top = Dimens.smallPadding)
                    )
                }
            }
        },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding)) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    onClick = onAttemptToSave,
                    isEnabled = stagedDns.isValid(),
                    text = stringResource(id = R.string.submit_button),
                )

                if (stagedDns is StagedDns.EditDns) {
                    PrimaryButton(
                        modifier = Modifier.fillMaxWidth(),
                        onClick = onRemove,
                        text = stringResource(id = R.string.remove_button)
                    )
                }

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    onClick = onDismiss,
                    text = stringResource(id = R.string.cancel)
                )
            }
        },
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
    )
}
