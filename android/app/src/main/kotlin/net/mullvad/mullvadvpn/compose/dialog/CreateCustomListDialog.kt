package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.component.CustomListNameTextField
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.compose.test.CREATE_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.usecase.customlists.CreateCustomListWithLocationsError
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewCreateCustomListDialog() {
    AppTheme { CreateCustomListDialog(state = CreateCustomListUiState()) }
}

@Preview
@Composable
private fun PreviewCreateCustomListDialogError() {
    AppTheme {
        CreateCustomListDialog(
            state =
                CreateCustomListUiState(
                    error =
                        CreateCustomListWithLocationsError.Create(
                            CreateCustomListError.CustomListAlreadyExists
                        )
                )
        )
    }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun CreateCustomList(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListResult.Created>,
    locationCode: GeographicLocationConstraint? = null
) {
    val vm: CreateCustomListDialogViewModel =
        koinViewModel(parameters = { parametersOf(locationCode) })
    LaunchedEffect(key1 = Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CreateCustomListDialogSideEffect.NavigateToCustomListLocationsScreen -> {
                    navigator.navigate(
                        CustomListLocationsDestination(
                            customListId = sideEffect.customListId,
                            newList = true
                        )
                    ) {
                        launchSingleTop = true
                    }
                }
                is CreateCustomListDialogSideEffect.ReturnWithResult -> {
                    backNavigator.navigateBack(result = sideEffect.result)
                }
            }
        }
    }
    val state by vm.uiState.collectAsStateWithLifecycle()
    CreateCustomListDialog(
        state = state,
        createCustomList = vm::createCustomList,
        onInputChanged = vm::clearError,
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun CreateCustomListDialog(
    state: CreateCustomListUiState,
    createCustomList: (String) -> Unit = {},
    onInputChanged: () -> Unit = {},
    onDismiss: () -> Unit = {}
) {

    val name = remember { mutableStateOf("") }
    val isValidName by remember { derivedStateOf { name.value.isNotBlank() } }

    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.create_new_list),
            )
        },
        text = {
            CustomListNameTextField(
                name = name.value,
                isValidName = isValidName,
                error = state.error?.errorString(),
                onSubmit = createCustomList,
                onValueChanged = {
                    name.value = it
                    onInputChanged()
                },
                modifier = Modifier.testTag(CREATE_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG)
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.create),
                onClick = { createCustomList(name.value) },
                isEnabled = isValidName
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}

@Composable
private fun CreateCustomListWithLocationsError.errorString() =
    stringResource(
        if (
            this is CreateCustomListWithLocationsError.Create &&
                this.error is CreateCustomListError.CustomListAlreadyExists
        ) {
            R.string.custom_list_error_list_exists
        } else {
            R.string.error_occurred
        }
    )
