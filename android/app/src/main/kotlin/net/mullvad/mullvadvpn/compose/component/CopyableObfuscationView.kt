package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment.Companion.CenterVertically
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.AnimatedIconButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewCopyableObfuscationView() {
    AppTheme { CopyableObfuscationView("1111222233334444", {}, modifier = Modifier.fillMaxWidth()) }
}

@Composable
fun CopyableObfuscationView(
    content: String,
    onCopyClicked: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var obfuscationEnabled by remember { mutableStateOf(true) }

    Row(verticalAlignment = CenterVertically, modifier = modifier) {
        AccountNumberView(
            accountNumber = content,
            obfuscateWithPasswordDots = obfuscationEnabled,
            modifier = Modifier.weight(1f),
        )
        AnimatedIconButton(
            defaultIcon = Icons.Outlined.Visibility,
            secondaryIcon = Icons.Outlined.VisibilityOff,
            defaultIconTint = MaterialTheme.colorScheme.onSurface,
            secondaryIconTint = MaterialTheme.colorScheme.onSurface,
            isToggleButton = true,
            contentDescription = stringResource(id = R.string.hide_account_number),
            onClick = { obfuscationEnabled = !obfuscationEnabled },
        )

        CopyAnimatedIconButton(onClick = { onCopyClicked(content) })
    }
}

@Composable
fun CopyAnimatedIconButton(onClick: () -> Unit) {
    AnimatedIconButton(
        defaultIcon = Icons.Default.ContentCopy,
        secondaryIcon = Icons.Default.Check,
        defaultIconTint = MaterialTheme.colorScheme.onSurface,
        secondaryIconTint = MaterialTheme.colorScheme.tertiary,
        isToggleButton = false,
        contentDescription = stringResource(id = R.string.copy_account_number),
        onClick = onClick,
    )
}
