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
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
fun PreviewCreateCustomListDialog() {
    AppTheme { CreateCustomListDialog() }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun CreateCustomList(backNavigator: ResultBackNavigator<String>) {
    val vm: CreateCustomListDialogViewModel = koinViewModel()
    val showError = remember { mutableStateOf(false) }
    LaunchedEffect(key1 = Unit) {
        vm.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CreateCustomListDialogSideEffect.NavigateToCustomListScreen -> {
                    backNavigator.navigateBack(sideEffect.customListId)
                }
                CreateCustomListDialogSideEffect.CreateCustomListError -> {
                    showError.value = true
                }
            }
        }
    }
    CreateCustomListDialog(
        showError = showError.value,
        createCustomList = vm::createCustomList,
        onInputChanged = { showError.value = false },
        onDismiss = backNavigator::navigateBack
    )
}

@Composable
fun CreateCustomListDialog(
    showError: Boolean = false,
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
                text = stringResource(id = R.string.create),
                onClick = { createCustomList(name.value) }
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}
