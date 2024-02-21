package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog() {
    AppTheme { DeleteCustomListConfirmationDialog(EmptyResultBackNavigator(), "My Custom List") }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun DeleteCustomListConfirmationDialog(navigator: ResultBackNavigator<Boolean>, name: String) {
    AlertDialog(
        onDismissRequest = { navigator.navigateBack() },
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
            NegativeButton(
                onClick = { navigator.navigateBack(result = true) },
                text = stringResource(id = R.string.delete_list)
            )
        },
        confirmButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = { navigator.navigateBack(result = false) },
                text = stringResource(id = R.string.cancel)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
