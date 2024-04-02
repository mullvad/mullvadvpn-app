package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.component.CustomListNameTextField
import net.mullvad.mullvadvpn.compose.state.UpdateCustomListUiState
import net.mullvad.mullvadvpn.compose.test.EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.ModifyCustomListError
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewEditCustomListNameDialog() {
    AppTheme { EditCustomListNameDialog(UpdateCustomListUiState()) }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun EditCustomListName(
    backNavigator: ResultBackNavigator<CustomListResult.Renamed>,
    customListId: CustomListId,
    initialName: String
) {
    val vm: EditCustomListNameDialogViewModel =
        koinViewModel(parameters = { parametersOf(customListId, initialName) })
    LaunchedEffectCollect(vm.uiSideEffect) { sideEffect ->
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
        onInputChanged = vm::clearError,
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun EditCustomListNameDialog(
    state: UpdateCustomListUiState,
    updateName: (String) -> Unit = {},
    onInputChanged: () -> Unit = {},
    onDismiss: () -> Unit = {}
) {
    val name = remember { mutableStateOf(state.name) }
    val isValidName by remember { derivedStateOf { name.value.isNotBlank() } }

    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.update_list_name),
            )
        },
        text = {
            CustomListNameTextField(
                name = name.value,
                isValidName = isValidName,
                error = state.error?.errorString(),
                onSubmit = updateName,
                onValueChanged = {
                    name.value = it
                    onInputChanged()
                },
                modifier = Modifier.testTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG)
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.save),
                onClick = { updateName(name.value) },
                isEnabled = isValidName
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}

@Composable
private fun ModifyCustomListError.errorString() =
    stringResource(
        when (this) {
            is UpdateCustomListError.NameAlreadyExists -> R.string.custom_list_error_list_exists
            GetCustomListError,
            is UpdateCustomListError.Unknown -> R.string.error_occurred
        }
    )
