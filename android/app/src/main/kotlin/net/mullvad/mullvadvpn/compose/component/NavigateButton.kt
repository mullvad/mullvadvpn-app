package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun NavigateBackIconButton(modifier: Modifier = Modifier, onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack, modifier = modifier) {
        Icon(
            imageVector = Icons.AutoMirrored.Default.ArrowBack,
            contentDescription = stringResource(id = R.string.back),
        )
    }
}

@Composable
fun NavigateBackDownIconButton(onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack) {
        Icon(
            imageVector = Icons.Default.ArrowDownward,
            contentDescription = stringResource(id = R.string.back),
        )
    }
}

@Composable
fun NavigateCloseIconButton(onNavigateClose: () -> Unit) {
    IconButton(onClick = onNavigateClose) {
        Icon(
            imageVector = Icons.Default.Close,
            contentDescription = stringResource(id = R.string.close),
        )
    }
}
