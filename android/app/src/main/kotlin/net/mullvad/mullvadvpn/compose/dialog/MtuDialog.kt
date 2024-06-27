package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.MtuTextField
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.viewmodel.MtuDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.MtuDialogUiState
import net.mullvad.mullvadvpn.viewmodel.MtuDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewMtuDialog() {
    AppTheme { MtuDialog(EmptyResultBackNavigator()) }
}

data class MtuNavArgs(val initialMtu: Mtu? = null)

@Destination<RootGraph>(style = DestinationStyle.Dialog::class, navArgs = MtuNavArgs::class)
@Composable
fun MtuDialog(navigator: ResultBackNavigator<Boolean>) {
    val viewModel = koinViewModel<MtuDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    LaunchedEffectCollect(viewModel.uiSideEffect) {
        when (it) {
            MtuDialogSideEffect.Complete -> navigator.navigateBack(result = true)
            MtuDialogSideEffect.Error -> navigator.navigateBack(result = false)
        }
    }
    MtuDialog(
        uiState,
        onInputChanged = viewModel::onInputChanged,
        onSaveMtu = viewModel::onSaveClick,
        onResetMtu = viewModel::onRestoreClick,
        onDismiss = dropUnlessResumed { navigator.navigateBack() })
}

@Composable
fun MtuDialog(
    state: MtuDialogUiState,
    onInputChanged: (String) -> Unit,
    onSaveMtu: (String) -> Unit,
    onResetMtu: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = stringResource(id = R.string.wireguard_mtu),
                color = MaterialTheme.colorScheme.onBackground,
            )
        },
        text = {
            Column {
                MtuTextField(
                    value = state.mtuInput,
                    onValueChanged = onInputChanged,
                    onSubmit = onSaveMtu,
                    isEnabled = true,
                    placeholderText = stringResource(R.string.enter_value_placeholder),
                    maxCharLength = 4,
                    isValidValue = state.isValidInput,
                    modifier = Modifier.fillMaxWidth())

                Text(
                    text =
                        stringResource(
                            id = R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaDescription),
                    modifier = Modifier.padding(top = Dimens.smallPadding))
            }
        },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing)) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    isEnabled = state.isValidInput,
                    text = stringResource(R.string.submit_button),
                    onClick = { onSaveMtu(state.mtuInput) })

                if (state.showResetToDefault) {
                    NegativeButton(
                        modifier = Modifier.fillMaxWidth(),
                        text = stringResource(R.string.reset_to_default_button),
                        onClick = onResetMtu)
                }

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(R.string.cancel),
                    onClick = onDismiss)
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
    )
}
