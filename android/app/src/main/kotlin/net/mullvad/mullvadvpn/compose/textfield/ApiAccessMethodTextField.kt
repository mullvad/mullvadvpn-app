package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun ApiAccessMethodTextField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    labelText: String?,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    errorText: String?,
    onSubmit: (String) -> Unit,
) {
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = onValueChanged,
        onSubmit = onSubmit,
        labelText = labelText,
        placeholderText = null,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        maxCharLength = maxCharLength,
        supportingText = errorText?.let { { ErrorSupportingText(errorText) } },
        colors = apiAccessTextFieldColors(),
        modifier = modifier.padding(vertical = Dimens.miniPadding),
    )
}

@Composable
private fun ErrorSupportingText(text: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.padding(top = Dimens.miniPadding)
    ) {
        Image(
            painter = painterResource(id = R.drawable.icon_alert),
            contentDescription = null,
            modifier = Modifier.size(Dimens.smallIconSize)
        )
        Text(
            text = text,
            color = MaterialTheme.colorScheme.onSecondary,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.padding(horizontal = Dimens.smallPadding)
        )
    }
}
