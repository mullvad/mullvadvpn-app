package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.textfield.MtuTextField
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.MtuDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.MtuDialogUiState
import net.mullvad.mullvadvpn.viewmodel.MtuDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewMtuDialog() {
    AppTheme {
        MtuDialog(
            state =
                MtuDialogUiState(mtuInput = "1300", isValidInput = true, showResetToDefault = true),
            onInputChanged = {},
            onSaveMtu = {},
            onResetMtu = {},
            onDismiss = {},
        )
    }
}

data class MtuNavArgs(val initialMtu: Mtu? = null)

@Destination<RootGraph>(style = DestinationStyle.Dialog::class, navArgs = MtuNavArgs::class)
@Composable
fun Mtu(navigator: ResultBackNavigator<Boolean>) {
    val viewModel = koinViewModel<MtuDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            MtuDialogSideEffect.Complete -> navigator.navigateBack(result = true)
            MtuDialogSideEffect.Error -> navigator.navigateBack(result = false)
        }
    }
    MtuDialog(
        state = uiState,
        onInputChanged = viewModel::onInputChanged,
        onSaveMtu = viewModel::onSaveClick,
        onResetMtu = viewModel::onRestoreClick,
        onDismiss = dropUnlessResumed { navigator.navigateBack() },
    )
}

@Composable
fun MtuDialog(
    state: MtuDialogUiState,
    onInputChanged: (String) -> Unit,
    onSaveMtu: (String) -> Unit,
    onResetMtu: () -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title = stringResource(id = R.string.mtu),
        message = stringResource(id = R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE),
        confirmButtonEnabled = state.isValidInput,
        onBack = onDismiss,
        onConfirm = { onSaveMtu(state.mtuInput) },
        onReset =
            if (state.showResetToDefault) {
                onResetMtu
            } else {
                null
            },
        input = {
            MtuTextField(
                value = state.mtuInput,
                onValueChanged = onInputChanged,
                onSubmit = onSaveMtu,
                isEnabled = true,
                placeholderText = stringResource(R.string.enter_value_placeholder),
                maxCharLength = 4,
                isValidValue = state.isValidInput,
                modifier = Modifier.fillMaxWidth(),
            )
        },
    )
}
