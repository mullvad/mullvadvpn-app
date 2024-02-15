package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
fun PreviewEditCustomListNameDialog() {
    AppTheme { EditCustomListNameDialog("Custom List Name") }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun EditCustomListName(navigator: DestinationsNavigator, customListId: String, name: String) {
    val vm: EditCustomListNameDialogViewModel =
        koinViewModel(parameters = { parametersOf(customListId) })
    val showError = remember { mutableStateOf(false) }
    LaunchedEffect(key1 = Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is EditCustomListNameDialogSideEffect.CloseScreen -> {
                    navigator.navigateUp()
                }
                EditCustomListNameDialogSideEffect.UpdateNameError -> {
                    showError.value = true
                }
            }
        }
    }
    EditCustomListNameDialog(
        name = name,
        showError = showError.value,
        updateName = vm::updateCustomListName,
        onInputChanged = { showError.value = false },
        onDismiss = navigator::navigateUp
    )
}

@Composable
fun EditCustomListNameDialog(
    name: String,
    showError: Boolean = false,
    updateName: (String) -> Unit = {},
    onInputChanged: () -> Unit = {},
    onDismiss: () -> Unit = {}
) {
    val input = remember { mutableStateOf(name) }
    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.create_new_list),
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
                    onSubmit = updateName,
                    keyboardType = KeyboardType.Text,
                    placeholderText = "",
                    isValidValue = input.value.isNotBlank(),
                    isDigitsOnlyAllowed = false
                )
                if (showError) {
                    Spacer(modifier = Modifier.height(Dimens.smallPadding))
                    Text(
                        text = stringResource(id = R.string.error_occurred),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.save),
                onClick = { updateName(input.value) }
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}
