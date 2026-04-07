package net.mullvad.mullvadvpn.lib.ui.component.button

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaVisible

@Composable
fun SearchButton(onClick: () -> Unit, enabled: Boolean) {
    IconButton(onClick = onClick) {
        Icon(
            imageVector = Icons.Rounded.Search,
            contentDescription = stringResource(id = R.string.search),
            tint =
                MaterialTheme.colorScheme.onSurface.copy(
                    alpha = if (enabled) AlphaVisible else AlphaDisabled
                ),
        )
    }
}
