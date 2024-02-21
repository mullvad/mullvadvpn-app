package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
fun PreviewCreateCustomListDialog() {
    AppTheme { CreateCustomListDialog(uiState = CreateCustomListUiState()) }
}

@Preview
@Composable
fun PreviewCreateCustomListDialogError() {
    AppTheme {
        CreateCustomListDialog(
            uiState = CreateCustomListUiState(error = CustomListsError.CustomListExists)
        )
    }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun CreateCustomList(backNavigator: ResultBackNavigator<String>) {
    val vm: CreateCustomListDialogViewModel = koinViewModel()
    LaunchedEffect(key1 = Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CreateCustomListDialogSideEffect.NavigateToCustomListScreen -> {
                    backNavigator.navigateBack(sideEffect.customListId)
                }
            }
        }
    }
    val uiState = vm.uiState.collectAsState().value
    CreateCustomListDialog(
        uiState = uiState,
        createCustomList = vm::createCustomList,
        onInputChanged = vm::clearError,
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun CreateCustomListDialog(
    uiState: CreateCustomListUiState,
    createCustomList: (String) -> Unit = {},
    onInputChanged: () -> Unit = {},
    onDismiss: () -> Unit = {}
) {
    val name = remember { mutableStateOf("") }
    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.create_new_list),
            )
        },
        text = {
            Column {
                CustomTextField(
                    value = name.value,
                    onValueChanged = {
                        name.value = it
                        onInputChanged()
                    },
                    onSubmit = createCustomList,
                    keyboardType = KeyboardType.Text,
                    placeholderText = "",
                    isValidValue = name.value.isNotBlank(),
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
                    }
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.create),
                onClick = { createCustomList(name.value) },
                isEnabled = name.value.isNotBlank()
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}
