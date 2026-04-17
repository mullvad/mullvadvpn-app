package net.mullvad.mullvadvpn.feature.vpnsettings.impl.mtu

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InputDialog
import net.mullvad.mullvadvpn.lib.ui.component.textfield.MtuTextField
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

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

@Composable
fun Mtu(navArgs: MtuNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<MtuDialogViewModel> { parametersOf(navArgs) }

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            MtuDialogSideEffect.Complete -> navigator.goBack(result = MtuNavResult(true))
            MtuDialogSideEffect.Error -> navigator.goBack(result = MtuNavResult(false))
        }
    }
    MtuDialog(
        state = uiState,
        onInputChanged = viewModel::onInputChanged,
        onSaveMtu = viewModel::onSaveClick,
        onResetMtu = viewModel::onRestoreClick,
        onDismiss = dropUnlessResumed { navigator.goBack() },
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
        message =
            AnnotatedString(
                stringResource(id = R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE)
            ),
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
