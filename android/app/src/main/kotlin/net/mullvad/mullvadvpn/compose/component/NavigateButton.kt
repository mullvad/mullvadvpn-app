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

@Composable
fun NavigateBackIconButton(modifier: Modifier = Modifier, onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack, modifier = modifier) {
        Icon(imageVector = Icons.AutoMirrored.Default.ArrowBack, contentDescription = null)
    }
}

@Composable
fun NavigateBackDownIconButton(onNavigateBack: () -> Unit) {
    IconButton(onClick = onNavigateBack) {
        Icon(imageVector = Icons.Default.ArrowDownward, contentDescription = null)
    }
}

@Composable
fun NavigateCloseIconButton(onNavigateClose: () -> Unit) {
    IconButton(onClick = onNavigateClose) {
        Icon(imageVector = Icons.Default.Close, contentDescription = null)
    }
}
