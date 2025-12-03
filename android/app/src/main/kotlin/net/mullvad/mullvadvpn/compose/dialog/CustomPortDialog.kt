package net.mullvad.mullvadvpn.compose.dialog

import android.os.Parcelable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.util.asString
import net.mullvad.mullvadvpn.viewmodel.CustomPortDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.CustomPortDialogViewModel
import org.koin.androidx.compose.koinViewModel

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
data class CustomPortDialogNavArgs(
    val portType: PortType,
    val allowedPortRanges: List<PortRange>,
    val customPort: Port?,
) : Parcelable

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun CustomPort(navArg: CustomPortDialogNavArgs, backNavigator: ResultBackNavigator<Port?>) {
    val viewModel = koinViewModel<CustomPortDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is CustomPortDialogSideEffect.Success -> backNavigator.navigateBack(it.port)
        }
    }

    val title =
        when (navArg.portType) {
            PortType.Udp2Tcp -> stringResource(R.string.udp_over_tcp)
            PortType.Shadowsocks -> stringResource(R.string.shadowsocks)
            PortType.Wireguard -> stringResource(R.string.wireguard)
            PortType.Lwo -> stringResource(R.string.lwo)
        }

    CustomPortDialog(
        title = title,
        portInput = uiState.portInput,
        isValidInput = uiState.isValidInput,
        showResetToDefault = uiState.showResetToDefault,
        allowedPortRanges = uiState.allowedPortRanges,
        onInputChanged = viewModel::onInputChanged,
        onSavePort = viewModel::onSaveClick,
        onResetPort = viewModel::onResetClick,
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() },
    )
}

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
        message =
            stringResource(
                id = R.string.custom_port_dialog_valid_ranges,
                allowedPortRanges.asString(),
            ),
        confirmButtonEnabled = isValidInput,
        confirmButtonText = stringResource(id = R.string.custom_port_dialog_submit),
        onResetButtonText = stringResource(R.string.custom_port_dialog_remove),
        onBack = onDismiss,
        onConfirm = { onSavePort(portInput) },
        onReset = if (showResetToDefault) onResetPort else null,
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
    )
}
