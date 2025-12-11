package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewInputDialog() {
    AppTheme {
        InputDialog(
            title = "Input here",
            message = AnnotatedString("Lorem ipsum"),
            onBack = {},
            onConfirm = {},
            onReset = {},
            input = {
                CustomTextField(
                    value = "input",
                    keyboardType = KeyboardType.Text,
                    onValueChanged = {},
                    onSubmit = {},
                    placeholder = { Text("Placeholder") },
                    isValidValue = true,
                    isDigitsOnlyAllowed = false,
                )
            },
        )
    }
}

@Suppress("ComposableLambdaParameterNaming")
@Composable
fun InputDialog(
    title: String,
    message: AnnotatedString? = null,
    confirmButtonEnabled: Boolean = true,
    confirmButtonText: String = stringResource(R.string.submit_button),
    onResetButtonText: String = stringResource(R.string.reset_to_default_button),
    messageTextColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    onBack: () -> Unit,
    onConfirm: () -> Unit,
    onReset: (() -> Unit)? = null,
    input: @Composable ColumnScope.() -> Unit,
) {
    AlertDialog(
        onDismissRequest = onBack,
        title = { Text(text = title, color = MaterialTheme.colorScheme.onSurface) },
        text = {
            Column {
                input()

                message?.let {
                    Text(
                        text = message,
                        style = MaterialTheme.typography.bodySmall,
                        color = messageTextColor,
                        modifier = Modifier.padding(top = Dimens.smallPadding),
                    )
                }
            }
        },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing)) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    isEnabled = confirmButtonEnabled,
                    text = confirmButtonText,
                    onClick = onConfirm,
                )

                if (onReset != null) {
                    NegativeButton(
                        modifier = Modifier.fillMaxWidth(),
                        text = onResetButtonText,
                        onClick = onReset,
                    )
                }

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(R.string.cancel),
                    onClick = onBack,
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
    )
}
