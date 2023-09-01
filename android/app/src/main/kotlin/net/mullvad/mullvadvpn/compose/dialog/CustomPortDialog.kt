package net.mullvad.mullvadvpn.compose.dialog

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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.lib.theme.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.util.asString
import net.mullvad.mullvadvpn.util.isPortInValidRanges

@Preview
@Composable
fun PreviewCustomPortDialog() {
    AppTheme {
        CustomPortDialog(
            onSave = {},
            onReset = {},
            customPort = "",
            allowedPortRanges = listOf(PortRange(10, 10), PortRange(40, 50)),
            showReset = true,
            onDismissRequest = {}
        )
    }
}

@Composable
fun CustomPortDialog(
    customPort: String,
    allowedPortRanges: List<PortRange>,
    showReset: Boolean,
    onSave: (customPortString: String) -> Unit,
    onReset: () -> Unit,
    onDismissRequest: () -> Unit
) {
    val port = remember { mutableStateOf(customPort) }

    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.custom_port_dialog_title),
                style = MaterialTheme.typography.headlineSmall
            )
        },
        confirmButton = {
            Column {
                ActionButton(
                    text = stringResource(id = R.string.custom_port_dialog_submit),
                    onClick = { onSave(port.value) },
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                            disabledContentColor =
                                MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaInactive),
                            disabledContainerColor =
                                MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled)
                        ),
                    isEnabled =
                        port.value.isNotEmpty() &&
                            allowedPortRanges.isPortInValidRanges(port.value.toIntOrNull() ?: 0)
                )
                if (showReset) {
                    ActionButton(
                        text = stringResource(R.string.custom_port_dialog_remove),
                        onClick = { onReset() },
                        modifier = Modifier.padding(top = Dimens.mediumPadding),
                        colors =
                            ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.error,
                                contentColor = MaterialTheme.colorScheme.onError,
                            )
                    )
                }
                ActionButton(
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                    text = stringResource(id = R.string.cancel),
                    modifier = Modifier.padding(top = Dimens.mediumPadding),
                    onClick = onDismissRequest
                )
            }
        },
        text = {
            Column {
                CustomPortTextField(
                    value = port.value,
                    onSubmit = { input ->
                        if (
                            input.isNotEmpty() &&
                                allowedPortRanges.isPortInValidRanges(input.toIntOrNull() ?: 0)
                        ) {
                            onSave(input)
                        }
                    },
                    onValueChanged = { input -> port.value = input },
                    isValidValue =
                        port.value.isNotEmpty() &&
                            allowedPortRanges.isPortInValidRanges(port.value.toIntOrNull() ?: 0),
                    maxCharLength = 5,
                    modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG)
                )
                Spacer(modifier = Modifier.height(Dimens.smallPadding))
                Text(
                    text =
                        stringResource(
                            id = R.string.custom_port_dialog_valid_ranges,
                            allowedPortRanges.asString()
                        ),
                    color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaDescription),
                    style = MaterialTheme.typography.bodySmall
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismissRequest
    )
}
