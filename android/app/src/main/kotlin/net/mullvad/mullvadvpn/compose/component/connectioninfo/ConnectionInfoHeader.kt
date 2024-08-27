package net.mullvad.mullvadvpn.compose.component.connectioninfo

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun ConnectionInfoHeader(text: String, modifier: Modifier = Modifier) {
    Text(
        modifier = modifier.padding(top = Dimens.smallPadding),
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        overflow = TextOverflow.Ellipsis,
    )
}
