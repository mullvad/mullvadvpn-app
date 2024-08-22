package net.mullvad.mullvadvpn.compose.dialog

import android.os.Parcelable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.util.asString
import net.mullvad.mullvadvpn.util.inAnyOf

@Preview
@Composable
private fun PreviewWireguardCustomPortDialog() {
    AppTheme {
        WireguardCustomPort(
            WireguardCustomPortNavArgs(
                customPort = null,
                allowedPortRanges = listOf(PortRange(10..10), PortRange(40..50)),
            ),
            EmptyResultBackNavigator()
        )
    }
}

@Parcelize
data class WireguardCustomPortNavArgs(
    val customPort: Port?,
    val allowedPortRanges: List<PortRange>,
) : Parcelable

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun WireguardCustomPort(
    navArg: WireguardCustomPortNavArgs,
    backNavigator: ResultBackNavigator<Port?>,
) {
    WireguardCustomPortDialog(
        initialPort = navArg.customPort,
        allowedPortRanges = navArg.allowedPortRanges,
        onSave = { port -> backNavigator.navigateBack(port) },
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun WireguardCustomPortDialog(
    initialPort: Port?,
    allowedPortRanges: List<PortRange>,
    onSave: (Port?) -> Unit,
    onDismiss: () -> Unit
) {
    val port = remember { mutableStateOf(initialPort?.value?.toString() ?: "") }
    val isValidPort = port.value.toPortOrNull()?.inAnyOf(allowedPortRanges) ?: false

    InputDialog(
        title = stringResource(id = R.string.custom_port_dialog_title),
        input = {
            CustomPortTextField(
                value = port.value,
                onSubmit = { input ->
                    if (isValidPort) {
                        onSave(input.toPortOrNull())
                    }
                },
                onValueChanged = { input -> port.value = input },
                isValidValue = isValidPort,
                maxCharLength = 5,
                modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).fillMaxWidth()
            )
        },
        message =
            stringResource(
                id = R.string.custom_port_dialog_valid_ranges,
                allowedPortRanges.asString()
            ),
        confirmButtonEnabled = isValidPort,
        confirmButtonText = stringResource(id = R.string.custom_port_dialog_submit),
        onResetButtonText = stringResource(R.string.custom_port_dialog_remove),
        onBack = onDismiss,
        onReset =
            if (initialPort != null) {
                { onSave(null) }
            } else {
                null
            },
        onConfirm = { onSave(port.value.toPortOrNull()) }
    )
}

private fun String.toPortOrNull() = toIntOrNull()?.let { Port(it) }
