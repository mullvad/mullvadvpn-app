package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import net.mullvad.mullvadvpn.R

@Composable
fun InfoIconButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onPrimary
) {
    IconButton(modifier = modifier, onClick = onClick) {
        Icon(
            painter = painterResource(id = R.drawable.icon_info),
            contentDescription = contentDescription,
            tint = iconTint
        )
    }
}
