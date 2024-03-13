package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import net.mullvad.mullvadvpn.R

@Composable
fun InfoIconButton(onClick: () -> Unit, modifier: Modifier = Modifier) {
    IconButton(modifier = modifier, onClick = onClick) {
        Icon(
            painter = painterResource(id = R.drawable.icon_info),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onPrimary
        )
    }
}
