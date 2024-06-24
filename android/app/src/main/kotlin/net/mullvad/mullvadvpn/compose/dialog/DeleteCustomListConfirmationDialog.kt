package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.state.DeleteCustomListUiState
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog() {
    AppTheme {
        DeleteCustomListConfirmationDialog(
            state = DeleteCustomListUiState(null),
            name = CustomListName.fromString("My Custom List")
        )
    }
}

@Composable
@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
fun DeleteCustomList(
    navigator: ResultBackNavigator<Deleted>,
    customListId: CustomListId,
    name: CustomListName
) {
    val viewModel: DeleteCustomListConfirmationViewModel =
        koinViewModel(parameters = { parametersOf(customListId) })
    val state = viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffectCollect(viewModel.uiSideEffect) {
        when (it) {
            is DeleteCustomListConfirmationSideEffect.ReturnWithResult ->
                navigator.navigateBack(result = it.result)
        }
    }

    DeleteCustomListConfirmationDialog(
        state = state.value,
        name = name,
        onDelete = viewModel::deleteCustomList,
        onBack = dropUnlessResumed { navigator.navigateBack() }
    )
}

@Composable
fun DeleteCustomListConfirmationDialog(
    state: DeleteCustomListUiState,
    name: CustomListName,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {}
) {
    DeleteConfirmationDialog(
        onDelete = onDelete,
        onBack = onBack,
        message =
            stringResource(id = R.string.delete_custom_list_confirmation_description, name.value),
        errorMessage =
            if (state.deleteError != null) {
                stringResource(id = R.string.error_occurred)
            } else {
                null
            }
    )
}
