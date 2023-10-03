package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.res.painterResource
import net.mullvad.mullvadvpn.R

@Composable
fun NavigateBackIconButton(onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack) {
        Icon(painter = painterResource(id = R.drawable.icon_back), contentDescription = null)
    }
}

@Composable
fun NavigateBackDownIconButton(onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack) {
        Icon(
            modifier = Modifier.rotate(-90f),
            painter = painterResource(id = R.drawable.icon_back),
            contentDescription = null
        )
    }
}
