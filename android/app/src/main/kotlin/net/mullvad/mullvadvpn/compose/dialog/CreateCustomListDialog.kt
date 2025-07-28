package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.compose.textfield.CustomListNameTextField
import net.mullvad.mullvadvpn.lib.model.CustomListAlreadyExists
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.tag.CREATE_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.usecase.customlists.CreateWithLocationsError
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewCreateCustomListDialog() {
    AppTheme {
        CreateCustomListDialog(
            state = CreateCustomListUiState(),
            createCustomList = {},
            onInputChanged = {},
            onDismiss = {},
        )
    }
}

@Preview
@Composable
private fun PreviewCreateCustomListDialogError() {
    AppTheme {
        CreateCustomListDialog(
            state =
                CreateCustomListUiState(
                    error = CreateWithLocationsError.Create(CustomListAlreadyExists)
                ),
            createCustomList = {},
            onInputChanged = {},
            onDismiss = {},
        )
    }
}

data class CreateCustomListNavArgs(val locationCode: GeoLocationId?)

@Composable
@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = CreateCustomListNavArgs::class,
)
fun CreateCustomList(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListActionResultData.Success.CreatedWithLocations>,
) {
    val vm: CreateCustomListDialogViewModel = koinViewModel()
    LaunchedEffect(key1 = Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CreateCustomListDialogSideEffect.NavigateToCustomListLocationsScreen -> {
                    navigator.navigate(
                        CustomListLocationsDestination(
                            customListId = sideEffect.customListId,
                            newList = true,
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
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() },
    )
}

@Composable
fun CreateCustomListDialog(
    state: CreateCustomListUiState,
    createCustomList: (String) -> Unit,
    onInputChanged: () -> Unit,
    onDismiss: () -> Unit,
) {
    val name = rememberSaveable { mutableStateOf("") }
    val isValidName = name.value.isNotBlank()

    InputDialog(
        title = stringResource(id = R.string.create_new_list),
        confirmButtonText = stringResource(id = R.string.create),
        confirmButtonEnabled = isValidName,
        input = {
            CustomListNameTextField(
                name = name.value,
                isValidName = isValidName,
                error = state.error?.errorString(),
                onSubmit = createCustomList,
                onValueChanged = {
                    name.value = it
                    onInputChanged()
                },
                modifier = Modifier.testTag(CREATE_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG),
            )
        },
        onBack = onDismiss,
        onConfirm = { createCustomList(name.value) },
    )
}

@Composable
private fun CreateWithLocationsError.errorString() =
    stringResource(
        if (this is CreateWithLocationsError.Create && this.error is CustomListAlreadyExists) {
            R.string.custom_list_error_list_exists
        } else {
            R.string.error_occurred
        }
    )
