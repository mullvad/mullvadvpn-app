package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.DeleteApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.DeleteApiAccessMethodConfirmationSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeleteApiAccessMethodConfirmationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewDeleteApiAccessMethodConfirmationDialog() {
    AppTheme { DeleteApiAccessMethodConfirmationDialog(state = DeleteApiAccessMethodUiState(null)) }
}

data class DeleteApiAccessMethodNavArgs(val apiAccessMethodId: ApiAccessMethodId)

@Composable
@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = DeleteApiAccessMethodNavArgs::class,
)
fun DeleteApiAccessMethodConfirmation(navigator: ResultBackNavigator<Boolean>) {
    val viewModel = koinViewModel<DeleteApiAccessMethodConfirmationViewModel>()
    val state = viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffectCollect(viewModel.uiSideEffect) {
        when (it) {
            is DeleteApiAccessMethodConfirmationSideEffect.Deleted ->
                navigator.navigateBack(result = true)
        }
    }

    DeleteApiAccessMethodConfirmationDialog(
        state = state.value,
        onDelete = viewModel::deleteApiAccessMethod,
        onBack = navigator::navigateBack,
    )
}

@Composable
fun DeleteApiAccessMethodConfirmationDialog(
    state: DeleteApiAccessMethodUiState,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {},
) {
    DeleteConfirmationDialog(
        onDelete = onDelete,
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
