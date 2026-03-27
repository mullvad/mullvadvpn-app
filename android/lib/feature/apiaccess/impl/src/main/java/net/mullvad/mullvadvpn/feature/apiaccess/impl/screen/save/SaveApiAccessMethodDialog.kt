package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.save

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
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.SaveApiAccessMethodNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.api.SaveApiAccessMethodNavResult
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewSaveApiAccessMethodDialog(
    @PreviewParameter(SaveApiAccessMethodUiStatePreviewParameterProvider::class)
    state: SaveApiAccessMethodUiState
) {
    AppTheme { SaveApiAccessMethodDialog(state = state, onCancel = {}, onSave = {}) }
}

@Composable
fun SaveApiAccessMethod(navArgs: SaveApiAccessMethodNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<SaveApiAccessMethodViewModel> { parametersOf(navArgs) }

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod ->
                navigator.goBack(SaveApiAccessMethodNavResult(false))
            SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod ->
                navigator.goBack(SaveApiAccessMethodNavResult(true))
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    SaveApiAccessMethodDialog(state = state, onCancel = navigator::goBack, onSave = viewModel::save)
}

@Composable
fun SaveApiAccessMethodDialog(
    state: SaveApiAccessMethodUiState,
    onCancel: () -> Unit,
    onSave: () -> Unit,
) {
    AlertDialog(
        icon = {
            when (val testingState = state.testingState) {
                is TestApiAccessMethodState.Result ->
                    Icon(
                        painter =
                            painterResource(
                                id =
                                    if (testingState is TestApiAccessMethodState.Result.Successful)
                                        R.drawable.icon_success
                                    else R.drawable.icon_fail
                            ),
                        contentDescription = null,
                    )
                TestApiAccessMethodState.Testing ->
                    MullvadCircularProgressIndicatorLarge(
                        modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG)
                    )
            }
        },
        text = {
            Text(text = state.descriptionText(), style = MaterialTheme.typography.labelLarge)
        },
        onDismissRequest = { /*Should not be able to dismiss*/ },
        confirmButton = {
            PrimaryButton(
                onClick = onCancel,
                text = stringResource(id = R.string.cancel),
                isEnabled =
                    state.testingState is TestApiAccessMethodState.Testing ||
                        state.testingState is TestApiAccessMethodState.Result.Failure,
                modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG),
            )
        },
        dismissButton = {
            if (state.testingState is TestApiAccessMethodState.Result.Failure) {
                PrimaryButton(
                    onClick = onSave,
                    text = stringResource(id = R.string.save),
                    modifier = Modifier.testTag(SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG),
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
        iconContentColor = Color.Unspecified,
    )
}

@Composable
private fun SaveApiAccessMethodUiState.descriptionText() =
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
