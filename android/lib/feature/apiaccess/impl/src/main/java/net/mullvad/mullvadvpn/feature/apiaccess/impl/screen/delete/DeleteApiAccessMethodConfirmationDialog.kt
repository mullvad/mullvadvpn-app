package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.delete

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.DeleteApiAccessMethodConfirmedNavResult
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.ui.component.dialog.NegativeConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewDeleteApiAccessMethodConfirmationDialog() {
    AppTheme { DeleteApiAccessMethodConfirmationDialog(state = DeleteApiAccessMethodUiState(null)) }
}

@Composable
fun DeleteApiAccessMethodConfirmation(apiAccessMethodId: ApiAccessMethodId, navigator: Navigator) {
    val viewModel =
        koinViewModel<DeleteApiAccessMethodConfirmationViewModel> {
            parametersOf(apiAccessMethodId)
        }
    val state = viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is DeleteApiAccessMethodConfirmationSideEffect.Deleted ->
                navigator.goBack(result = DeleteApiAccessMethodConfirmedNavResult)
        }
    }

    DeleteApiAccessMethodConfirmationDialog(
        state = state.value,
        onDelete = viewModel::deleteApiAccessMethod,
        onBack = navigator::goBack,
    )
}

@Composable
fun DeleteApiAccessMethodConfirmationDialog(
    state: DeleteApiAccessMethodUiState,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {},
) {
    NegativeConfirmationDialog(
        onConfirm = onDelete,
        onBack = onBack,
        message = stringResource(id = R.string.delete_method_question),
        errorMessage =
            if (state.deleteError != null) {
                stringResource(id = R.string.error_occurred)
            } else {
                null
            },
    )
}
