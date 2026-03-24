package net.mullvad.mullvadvpn.common.compose

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.core.scene.LocalSceneRole

@SuppressLint("ComposableNaming")
@Composable
fun unlessIsDetail(block: @Composable () -> Unit) {
    if (LocalSceneRole.current != ListDetailSceneStrategy.Role.Detail) {
        block()
    }
}

// If we are in portrait and then rotate to landscape which triggers the list-detail scene
// the list pane is already on the back stack, but the detail pane isn't so we need to push it.
@SuppressLint("ComposableNaming")
@Composable
inline fun <reified T : NavKey2> Navigator.navigateToDetailIfNeeded(detailKey: NavKey2) {
    LaunchedEffect(detailKey) {
        if (screenIsListDetailTargetWidth && backStack.last() is T) {
            navigate(detailKey)
        }
    }
}
