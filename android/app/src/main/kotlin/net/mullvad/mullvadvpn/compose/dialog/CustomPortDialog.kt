package net.mullvad.mullvadvpn.compose.dialog

import android.os.Parcelable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.util.asString

@Preview
@Composable
private fun PreviewWireguardCustomPortDialog() {
    AppTheme {
        CustomPortDialog(
            title = "Custom port",
            portInput = "",
            isValidInput = false,
            allowedPortRanges = listOf(PortRange(10..10), PortRange(40..50)),
            showResetToDefault = false,
            onInputChanged = {},
            onSavePort = {},
            onResetPort = {},
            onDismiss = {},
        )
    }
}

@Parcelize
data class CustomPortNavArgs(val customPort: Port?, val allowedPortRanges: List<PortRange>) :
    Parcelable

@Composable
fun CustomPortDialog(
    title: String,
    portInput: String,
    isValidInput: Boolean,
    allowedPortRanges: List<PortRange>,
    showResetToDefault: Boolean,
    onInputChanged: (String) -> Unit,
    onSavePort: (String) -> Unit,
    onResetPort: () -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title = title,
        input = {
            CustomPortTextField(
                value = portInput,
                onValueChanged = onInputChanged,
                onSubmit = onSavePort,
                isValidValue = isValidInput,
                maxCharLength = 5,
                modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).fillMaxWidth(),
            )
        },
        message =
            stringResource(
                id = R.string.custom_port_dialog_valid_ranges,
                allowedPortRanges.asString(),
            ),
        confirmButtonEnabled = isValidInput,
        confirmButtonText = stringResource(id = R.string.custom_port_dialog_submit),
        onResetButtonText = stringResource(R.string.custom_port_dialog_remove),
        onBack = onDismiss,
        onReset = if (showResetToDefault) onResetPort else null,
        onConfirm = { onSavePort(portInput) },
    )
}
