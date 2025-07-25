package net.mullvad.mullvadvpn.compose.dialog

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
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.EditCustomListNameUiState
import net.mullvad.mullvadvpn.compose.textfield.CustomListNameTextField
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.UnknownCustomListError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.tag.EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.usecase.customlists.RenameError
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewEditCustomListNameDialog() {
    AppTheme {
        EditCustomListNameDialog(
            state = EditCustomListNameUiState(),
            updateName = {},
            onInputChanged = {},
            onDismiss = {},
        )
    }
}

data class EditCustomListNameNavArgs(
    val customListId: CustomListId,
    val initialName: CustomListName,
)

@Composable
@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = EditCustomListNameNavArgs::class,
)
fun EditCustomListName(
    backNavigator: ResultBackNavigator<CustomListActionResultData.Success.Renamed>
) {
    val vm: EditCustomListNameDialogViewModel = koinViewModel()
    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is EditCustomListNameDialogSideEffect.ReturnWithResult -> {
                backNavigator.navigateBack(result = sideEffect.result)
            }
        }
    }

    val state by vm.uiState.collectAsStateWithLifecycle()
    EditCustomListNameDialog(
        state = state,
        updateName = vm::updateCustomListName,
        onInputChanged = vm::onNameChanged,
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() },
    )
}

@Composable
fun EditCustomListNameDialog(
    state: EditCustomListNameUiState,
    updateName: (String) -> Unit,
    onInputChanged: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title = stringResource(id = R.string.update_list_name),
        confirmButtonText = stringResource(id = R.string.save),
        confirmButtonEnabled = state.isValidName,
        input = {
            CustomListNameTextField(
                name = state.name,
                isValidName = state.isValidName,
                error = state.error?.errorString(),
                onSubmit = updateName,
                onValueChanged = onInputChanged,
                modifier = Modifier.testTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG),
            )
        },
        onBack = onDismiss,
        onConfirm = { updateName(state.name) },
    )
}

@Composable
private fun RenameError.errorString() =
    stringResource(
        when (error) {
            is NameAlreadyExists -> R.string.custom_list_error_list_exists
            is GetCustomListError,
            is UnknownCustomListError -> R.string.error_occurred
        }
    )
