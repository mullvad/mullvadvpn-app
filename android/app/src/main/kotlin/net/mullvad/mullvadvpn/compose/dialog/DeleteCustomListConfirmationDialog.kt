package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.text.HtmlCompat
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog() {
    AppTheme { DeleteCustomListConfirmationDialog("My Custom List") }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun DeleteCustomList(navigator: ResultBackNavigator<Boolean>, id: String, name: String) {
    val viewModel: DeleteCustomListConfirmationViewModel =
        koinViewModel(parameters = { parametersOf(id) })

    LaunchedEffect(Unit) {
        viewModel.uiSideEffect.collect {
            when (it) {
                is DeleteCustomListConfirmationSideEffect.CloseDialog ->
                    navigator.navigateBack(result = true)
            }
        }
    }

    DeleteCustomListConfirmationDialog(
        name = name,
        onDelete = viewModel::deleteCustomList,
        onBack = { navigator.navigateBack(result = false) }
    )
}

@Composable
fun DeleteCustomListConfirmationDialog(
    name: String,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {}
) {
    AlertDialog(
        onDismissRequest = onBack,
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = stringResource(id = R.string.remove_button),
                tint = Color.Unspecified
            )
        },
        text = {
            val htmlFormattedDialogText =
                HtmlCompat.fromHtml(
                    textResource(id = R.string.delete_custom_list_confirmation_description, name),
                    HtmlCompat.FROM_HTML_MODE_COMPACT
                )
            val annotatedText = htmlFormattedDialogText.toAnnotatedString(FontWeight.Bold)

            Text(
                text = annotatedText,
                color = MaterialTheme.colorScheme.onBackground,
                style = MaterialTheme.typography.bodySmall,
            )
        },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = onBack,
                text = stringResource(id = R.string.cancel)
            )
        },
        confirmButton = {
            NegativeButton(onClick = onDelete, text = stringResource(id = R.string.delete_list))
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
