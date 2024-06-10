package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.preview.SaveApiAccessMethodUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.SaveApiAccessMethodSideEffect
import net.mullvad.mullvadvpn.viewmodel.SaveApiAccessMethodViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewSaveApiAccessMethodDialog(
    @PreviewParameter(SaveApiAccessMethodUiStatePreviewParameterProvider::class)
    state: SaveApiAccessMethodUiState
) {
    AppTheme { SaveApiAccessMethodDialog(state = state) }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun SaveApiAccessMethod(
    backNavigator: ResultBackNavigator<Boolean>,
    id: ApiAccessMethodId?,
    name: ApiAccessMethodName,
    customProxy: ApiAccessMethodType.CustomProxy
) {
    val viewModel =
        koinViewModel<SaveApiAccessMethodViewModel>(
            parameters = { parametersOf(id, name, customProxy) }
        )

    LaunchedEffectCollect(sideEffect = viewModel.uiSideEffect) {
        when (it) {
            SaveApiAccessMethodSideEffect.Cancel -> backNavigator.navigateBack()
            SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod ->
                backNavigator.navigateBack(result = false)
            SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod ->
                backNavigator.navigateBack(result = true)
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    SaveApiAccessMethodDialog(state = state, onCancel = viewModel::cancel, onSave = viewModel::save)
}

@Composable
fun SaveApiAccessMethodDialog(
    state: SaveApiAccessMethodUiState,
    onCancel: () -> Unit = {},
    onSave: () -> Unit = {}
) {
    AlertDialog(
        icon = {
            when (val testingState = state.testingState) {
                is TestApiAccessMethodState.Result ->
                    Icon(
                        painter =
                            painterResource(
                                id =
                                    if (testingState.isSuccessful()) {
                                        R.drawable.icon_success
                                    } else {
                                        R.drawable.icon_fail
                                    }
                            ),
                        contentDescription = null
                    )
                TestApiAccessMethodState.Testing ->
                    MullvadCircularProgressIndicatorMedium(
                        modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG)
                    )
            }
        },
        title = { Text(text = state.text(), style = MaterialTheme.typography.headlineSmall) },
        onDismissRequest = { /*Should not be able to dismiss*/},
        confirmButton = {
            PrimaryButton(
                onClick = onCancel,
                text = stringResource(id = R.string.cancel),
                isEnabled =
                    state.testingState is TestApiAccessMethodState.Testing ||
                        state.testingState is TestApiAccessMethodState.Result.Failure,
                modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG)
            )
        },
        dismissButton = {
            if (state.testingState is TestApiAccessMethodState.Result.Failure) {
                PrimaryButton(
                    onClick = onSave,
                    text = stringResource(id = R.string.save),
                    modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG)
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        iconContentColor = Color.Unspecified,
    )
}

@Composable
private fun SaveApiAccessMethodUiState.text() =
    stringResource(
        id =
            when (testingState) {
                TestApiAccessMethodState.Testing -> R.string.verifying_api_method
                TestApiAccessMethodState.Result.Successful -> R.string.api_reachable_adding_method
                TestApiAccessMethodState.Result.Failure -> {
                    if (isSaving) {
                        R.string.adding_method
                    } else {
                        R.string.api_unreachable_save_anyway
                    }
                }
            }
    )
