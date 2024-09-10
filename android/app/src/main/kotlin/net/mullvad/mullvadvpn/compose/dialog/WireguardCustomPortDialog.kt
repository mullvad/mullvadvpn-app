package net.mullvad.mullvadvpn.compose.dialog

import android.os.Parcelable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.CustomPortTextField
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
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
fun WireguardCustomPort(navArg: WireguardCustomPortNavArgs, navigator: ResultBackNavigator<Port?>) {
    val viewModel = koinViewModel<WireguardCustomPortDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is WireguardCustomPortDialogSideEffect.Success -> navigator.navigateBack(it.port)
        }
    }

    WireguardCustomPortDialog(
        uiState,
        onInputChanged = viewModel::onInputChanged,
        onSavePort = viewModel::onSaveClick,
        onResetPort = viewModel::onResetClick,
        onDismiss = dropUnlessResumed { navigator.navigateBack() },
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
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(text = stringResource(id = R.string.custom_port_dialog_title)) },
        text = {
            Column {
                CustomPortTextField(
                    value = state.portInput,
                    onValueChanged = onInputChanged,
                    onSubmit = onSavePort,
                    isValidValue = state.isValidInput,
                    maxCharLength = 5,
                    modifier = Modifier.testTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).fillMaxWidth(),
                )
                Spacer(modifier = Modifier.height(Dimens.smallPadding))
                Text(
                    text =
                        stringResource(
                            id = R.string.custom_port_dialog_valid_ranges,
                            state.allowedPortRanges.asString(),
                        ),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing)) {
                PrimaryButton(
                    isEnabled = state.isValidInput,
                    text = stringResource(id = R.string.custom_port_dialog_submit),
                    onClick = { onSavePort(state.portInput) },
                )
                if (state.showResetToDefault) {
                    NegativeButton(
                        text = stringResource(R.string.custom_port_dialog_remove),
                        onClick = onResetPort,
                    )
                }
                PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
            }
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
    )
}
