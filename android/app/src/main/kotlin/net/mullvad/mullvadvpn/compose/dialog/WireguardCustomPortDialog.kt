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
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.util.asString
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogUiState
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewWireguardCustomPortDialog() {
    AppTheme {
        WireguardCustomPort(
            WireguardCustomPortNavArgs(
                customPort = null,
                allowedPortRanges = listOf(PortRange(10..10), PortRange(40..50)),
            ),
            EmptyResultBackNavigator(),
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
    @Suppress("UNUSED_PARAMETER") navArg: WireguardCustomPortNavArgs,
    backNavigator: ResultBackNavigator<Port?>,
) {
    val viewModel = koinViewModel<WireguardCustomPortDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is WireguardCustomPortDialogSideEffect.Success -> backNavigator.navigateBack(it.port)
        }
    }

    WireguardCustomPortDialog(
        uiState,
        onInputChanged = viewModel::onInputChanged,
        onSavePort = viewModel::onSaveClick,
        onResetPort = viewModel::onResetClick,
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() },
    )
}

@Composable
fun WireguardCustomPortDialog(
    state: WireguardCustomPortDialogUiState,
    onInputChanged: (String) -> Unit,
    onSavePort: (String) -> Unit,
    onResetPort: () -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title = stringResource(id = R.string.custom_port_dialog_title),
        input = {
            CustomPortTextField(
                value = state.portInput,
                onValueChanged = onInputChanged,
                onSubmit = onSavePort,
                isValidValue = state.isValidInput,
                maxCharLength = 5,
                modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).fillMaxWidth(),
            )
        },
        message =
            stringResource(
                id = R.string.custom_port_dialog_valid_ranges,
                state.allowedPortRanges.asString(),
            ),
        confirmButtonEnabled = state.isValidInput,
        confirmButtonText = stringResource(id = R.string.custom_port_dialog_submit),
        onResetButtonText = stringResource(R.string.custom_port_dialog_remove),
        onBack = onDismiss,
        onReset = if (state.showResetToDefault) onResetPort else null,
        onConfirm = { onSavePort(state.portInput) },
    )
}
