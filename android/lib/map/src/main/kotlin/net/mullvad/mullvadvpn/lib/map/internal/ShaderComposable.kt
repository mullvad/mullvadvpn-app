package net.mullvad.mullvadvpn.lib.map.internal

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import net.mullvad.mullvadvpn.lib.map.data.MapConfig
import net.mullvad.mullvadvpn.lib.map.data.MapViewState

