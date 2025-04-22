package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun LocationsEmptyText(searchTerm: String, isSearching: Boolean) {
    if (isSearching) {
        Text(
            text = textResource(R.string.search_location_empty_text, searchTerm),
            style = MaterialTheme.typography.labelMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(Dimens.screenVerticalMargin),
        )
    } else {
        Text(
            text = stringResource(R.string.no_locations_found),
            modifier = Modifier.padding(Dimens.screenVerticalMargin),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
