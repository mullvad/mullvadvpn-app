package net.mullvad.mullvadvpn.core.scene

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavMetadataKey
import androidx.navigation3.runtime.get
import androidx.navigation3.runtime.metadata
import androidx.navigation3.scene.OverlayScene
import androidx.navigation3.scene.Scene
import androidx.navigation3.scene.SceneStrategy
import androidx.navigation3.scene.SceneStrategyScope
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy.Companion.overlay

/** An [OverlayScene] that renders an [entry] as an overlay. */
@OptIn(ExperimentalMaterial3Api::class)
internal class SingleOverlayScene<T : Any>(
    override val key: T,
    override val previousEntries: List<NavEntry<T>>,
    override val overlaidEntries: List<NavEntry<T>>,
    private val entry: NavEntry<T>,
) : OverlayScene<T> {

    override val entries: List<NavEntry<T>> = listOf(entry)

    override val content: @Composable (() -> Unit) = { entry.Content() }
}

/**
 * A [SceneStrategy] that displays entries that have added [overlay] to their [NavEntry.metadata] as
 * an overlay.
 *
 * This strategy should always be added before any non-overlay scene strategies.
 */
@OptIn(ExperimentalMaterial3Api::class)
class SingleOverlaySceneStrategy<T : Any> : SceneStrategy<T> {

    override fun SceneStrategyScope<T>.calculateScene(entries: List<NavEntry<T>>): Scene<T>? {
        val lastEntry = entries.lastOrNull() ?: return null

        return lastEntry.metadata[BottomSheetKey]?.let {
            @Suppress("UNCHECKED_CAST")
            SingleOverlayScene(
                key = lastEntry.contentKey as T,
                previousEntries = entries.dropLast(1),
                overlaidEntries = entries.dropLast(1),
                entry = lastEntry,
            )
        }
    }

    companion object {
        /**
         * Function to be called on the [NavEntry.metadata] to mark this entry as something that
         * should be displayed as an overlay.
         */
        fun overlay() = metadata { put(BottomSheetKey, Unit) }

        object BottomSheetKey : NavMetadataKey<Unit>
    }
}
