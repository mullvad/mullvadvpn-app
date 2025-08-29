package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.DeleteCustomListUiState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog() {
    AppTheme {
        DeleteCustomListConfirmationDialog(
            state = DeleteCustomListUiState(CustomListName.fromString("My Custom List"), null),
            onDelete = {},
            onBack = {},
        )
    }
}

data class DeleteCustomListNavArgs(val customListId: CustomListId, val name: CustomListName)

@Composable
@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = DeleteCustomListNavArgs::class,
)
fun DeleteCustomList(navigator: ResultBackNavigator<CustomListActionResultData.Success.Deleted>) {
    val viewModel: DeleteCustomListConfirmationViewModel = koinViewModel()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is DeleteCustomListConfirmationSideEffect.ReturnWithResult ->
                navigator.navigateBack(result = it.result)
        }
    }

    DeleteCustomListConfirmationDialog(
        state = state,
        onDelete = viewModel::deleteCustomList,
        onBack = dropUnlessResumed { navigator.navigateBack() },
    )
}

@Composable
fun DeleteCustomListConfirmationDialog(
    state: DeleteCustomListUiState,
    onDelete: () -> Unit,
    onBack: () -> Unit,
) {
    NegativeConfirmationDialog(
        onConfirm = onDelete,
        onBack = onBack,
        message =
            stringResource(
                id = R.string.delete_custom_list_confirmation_description,
                state.name.value,
            ),
        errorMessage =
            if (state.deleteError != null) {
                stringResource(id = R.string.error_occurred)
            } else {
                null
            },
    )
}
