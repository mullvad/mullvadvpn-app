package net.mullvad.mullvadvpn.lib.ui.component.button

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Preview
@Composable
private fun PreviewSearchButton() {
    PreviewColumn {
        SearchButton(onClick = {})
        SearchButton(onClick = {}, enabled = false)
    }
}

@Composable
fun SearchButton(onClick: () -> Unit, enabled: Boolean = true) {
    IconButton(onClick = onClick, enabled = enabled) {
        Icon(
            imageVector = Icons.Rounded.Search,
            contentDescription = stringResource(id = R.string.search),
        )
    }
}
