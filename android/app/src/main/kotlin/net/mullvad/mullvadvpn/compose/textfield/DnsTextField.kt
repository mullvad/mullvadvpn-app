package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun DnsTextField(
    value: String,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = {},
    onSubmit: () -> Unit = {},
    placeholderText: String?,
    errorText: String?,
    isEnabled: Boolean = true,
    isValidValue: Boolean = true,
) {
    CustomTextField(
        value = value,
        keyboardType = KeyboardType.Text,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onSubmit = { onSubmit() },
        isEnabled = isEnabled,
        placeholderText = placeholderText,
        supportingText = errorText?.let { { ErrorSupportingText(errorText) } },
        maxCharLength = Int.MAX_VALUE,
        isDigitsOnlyAllowed = false,
        isValidValue = isValidValue,
    )
}

@Composable
private fun ErrorSupportingText(text: String) {
    Row(modifier = Modifier.padding(top = Dimens.miniPadding)) {
        Icon(
            imageVector = Icons.Default.Error,
            contentDescription = null,
            modifier = Modifier.size(Dimens.smallIconSize),
            tint = MaterialTheme.colorScheme.error,
        )
        Text(
            text = text,
            color = MaterialTheme.colorScheme.onSecondary,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.padding(horizontal = Dimens.smallPadding),
        )
    }
}
