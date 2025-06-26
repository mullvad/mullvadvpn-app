package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun EmptyRelayListText() {
    Text(
        text = stringResource(R.string.no_locations_found),
        modifier = Modifier.padding(Dimens.cellVerticalSpacing),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}
