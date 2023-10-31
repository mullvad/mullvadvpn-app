package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.MtuTextField
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.util.isValidMtu

@Preview
@Composable
private fun PreviewMtuDialog() {
    AppTheme {
        MtuDialog(mtuInitial = 1234, onSave = {}, onRestoreDefaultValue = {}, onDismiss = {})
    }
}

@Composable
fun MtuDialog(
    mtuInitial: Int?,
    onSave: (Int) -> Unit,
    onRestoreDefaultValue: () -> Unit,
    onDismiss: () -> Unit,
) {
    val mtu = remember { mutableStateOf(mtuInitial?.toString() ?: "") }

    val isValidMtu = mtu.value.toIntOrNull()?.isValidMtu() == true

    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = stringResource(id = R.string.wireguard_mtu),
                color = MaterialTheme.colorScheme.onBackground,
                style = MaterialTheme.typography.headlineSmall
            )
        },
        text = {
            Column {
                MtuTextField(
                    value = mtu.value,
                    onValueChanged = { newMtuValue -> mtu.value = newMtuValue },
                    onSubmit = { newMtuValue ->
                        val mtuInt = newMtuValue.toIntOrNull()
                        if (mtuInt?.isValidMtu() == true) {
                            onSave(mtuInt)
                        }
                    },
                    isEnabled = true,
                    placeholderText = stringResource(R.string.enter_value_placeholder),
                    maxCharLength = 4,
                    isValidValue = isValidMtu,
                    modifier = Modifier.fillMaxWidth()
                )

                Text(
                    text =
                        stringResource(
                            id = R.string.wireguard_mtu_footer,
                            MTU_MIN_VALUE,
                            MTU_MAX_VALUE
                        ),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaDescription),
                    modifier = Modifier.padding(top = Dimens.smallPadding)
                )
            }
        },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSeparation)) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    isEnabled = isValidMtu,
                    text = stringResource(R.string.submit_button),
                    onClick = {
                        val mtuInt = mtu.value.toIntOrNull()
                        if (mtuInt?.isValidMtu() == true) {
                            onSave(mtuInt)
                        }
                    }
                )

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(R.string.reset_to_default_button),
                    onClick = onRestoreDefaultValue
                )

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(R.string.cancel),
                    onClick = onDismiss
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
    )
}
