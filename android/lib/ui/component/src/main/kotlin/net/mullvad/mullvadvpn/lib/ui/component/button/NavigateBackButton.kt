package net.mullvad.mullvadvpn.lib.ui.component.button

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.ui.component.R

@Composable
fun NavigateBackIconButton(
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onNavigateBack: () -> Unit,
) {
    IconButton(onClick = onNavigateBack, enabled = enabled, modifier = modifier) {
        Icon(
            imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
            contentDescription = stringResource(id = R.string.back),
        )
    }
}

