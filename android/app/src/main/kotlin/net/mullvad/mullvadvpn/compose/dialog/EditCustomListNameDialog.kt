package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.communication.EditCustomListNameDialogRequest
import net.mullvad.mullvadvpn.compose.communication.EditCustomListNameDialogResult
import net.mullvad.mullvadvpn.compose.state.UpdateCustomListUiState
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
fun PreviewEditCustomListNameDialog() {
    AppTheme { EditCustomListNameDialog(UpdateCustomListUiState()) }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun EditCustomListName(
    backNavigator: ResultBackNavigator<EditCustomListNameDialogResult>,
    request: EditCustomListNameDialogRequest
) {
    val vm: EditCustomListNameDialogViewModel =
        koinViewModel(parameters = { parametersOf(request.customListId, request.name) })
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is EditCustomListNameDialogSideEffect.ReturnResult -> {
                    backNavigator.navigateBack(result = sideEffect.result)
                }
            }
        }
    }

    val uiState = vm.uiState.collectAsState().value
    EditCustomListNameDialog(
        uiState = uiState,
        updateName = vm::updateCustomListName,
        onInputChanged = vm::clearError,
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun EditCustomListNameDialog(
    uiState: UpdateCustomListUiState,
    updateName: (String) -> Unit = {},
    onInputChanged: () -> Unit = {},
    onDismiss: () -> Unit = {}
) {
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current
    val input = remember { mutableStateOf(uiState.name) }
    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.update_list_name),
            )
        },
        text = {
            Column {
                CustomTextField(
                    value = input.value,
                    onValueChanged = {
                        input.value = it
                        onInputChanged()
                    },
                    onSubmit = {
                        if (it.isNotBlank()) {
                            updateName(it)
                        }
                    },
                    keyboardType = KeyboardType.Text,
                    placeholderText = "",
                    isValidValue = uiState.error == null,
                    isDigitsOnlyAllowed = false,
                    supportingText = {
                        if (uiState.error != null) {
                            Text(
                                text =
                                    stringResource(
                                        id =
                                            if (
                                                uiState.error == CustomListsError.CustomListExists
                                            ) {
                                                R.string.custom_list_error_list_exists
                                            } else {
                                                R.string.error_occurred
                                            }
                                    ),
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }
                    },
                    modifier =
                        Modifier.focusRequester(focusRequester).onFocusChanged { focusState ->
                            if (focusState.hasFocus) {
                                keyboardController?.show()
                            }
                        }
                )
            }

            LaunchedEffect(Unit) { focusRequester.requestFocus() }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.save),
                onClick = { updateName(input.value) },
                isEnabled = input.value.isNotBlank()
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}
