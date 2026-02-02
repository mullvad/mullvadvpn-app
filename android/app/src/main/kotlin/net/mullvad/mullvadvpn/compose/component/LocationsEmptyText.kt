package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Composable
fun LocationsEmptyText(searchTerm: String) {
    Text(
        text = textResource(R.string.search_location_empty_text, searchTerm),
        style = MaterialTheme.typography.bodyMedium,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.padding(Dimens.cellVerticalSpacing),
    )
}
