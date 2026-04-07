package net.mullvad.mullvadvpn.lib.ui.component.button

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.ui.component.R

@Composable
fun NavigateCloseIconButton(onNavigateClose: () -> Unit) {
    IconButton(onClick = onNavigateClose) {
        Icon(
            imageVector = Icons.Rounded.Close,
            contentDescription = stringResource(id = R.string.close),
        )
    }
}
