package net.mullvad.mullvadvpn.feature.anticensorship.impl.customport

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.CustomPortNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.api.CustomPortNavResult
import net.mullvad.mullvadvpn.lib.common.util.asString
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InputDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewWireguardCustomPortDialog() {
    AppTheme {
        CustomPortDialog(
            title = "Custom port",
            portInput = "",
            inputError = null,
            allowedPortRanges = listOf(PortRange(10..10), PortRange(40..50)),
            recommendedPortRanges = listOf(PortRange(10..10)),
            showResetToDefault = false,
            onInputChanged = {},
            onSavePort = {},
            onResetPort = {},
            onDismiss = {},
        )
    }
}

@Composable
fun CustomPort(navArg: CustomPortNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<CustomPortDialogViewModel> { parametersOf(navArg) }

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is CustomPortDialogSideEffect.Success ->
                navigator.goBack(result = CustomPortNavResult(it.port))
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
        inputError = uiState.portInputError,
        showResetToDefault = uiState.showResetToDefault,
        allowedPortRanges = uiState.allowedPortRanges,
        recommendedPortRanges = uiState.recommendedPortRanges,
        onInputChanged = viewModel::onInputChanged,
        onSavePort = viewModel::onSaveClick,
        onResetPort = viewModel::onResetClick,
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun CustomPortDialog(
    title: String,
    portInput: String,
    inputError: ParsePortError?,
    allowedPortRanges: List<PortRange>,
    recommendedPortRanges: List<PortRange>,
    showResetToDefault: Boolean,
    onInputChanged: (String) -> Unit,
    onSavePort: (String) -> Unit,
    onResetPort: () -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title = title,
        message =
            buildAnnotatedString {
                appendLine(
                    stringResource(
                        id = R.string.custom_port_dialog_valid_ranges,
                        allowedPortRanges.asString(),
                    )
                )
                if (recommendedPortRanges.isNotEmpty()) {
                    append(annotatedStringResource(R.string.custom_port_recommended_range_first))
                    append(
                        stringResource(
                            id = R.string.custom_port_recommended_range_second,
                            recommendedPortRanges.asString(),
                        )
                    )
                }
            },
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
                isValidValue = inputError == null,
                maxCharLength = 5,
                modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).fillMaxWidth(),
                errorText =
                    when (inputError) {
                        ParsePortError.Blank ->
                            stringResource(R.string.custom_port_input_error_blank)
                        is ParsePortError.NotANumber,
                        is ParsePortError.OutOfRange ->
                            stringResource(R.string.custom_port_input_error_out_of_range)
                        null -> null
                    },
            )
        },
    )
}
