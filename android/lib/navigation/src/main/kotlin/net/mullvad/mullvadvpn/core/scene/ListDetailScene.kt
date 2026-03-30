package net.mullvad.mullvadvpn.core.scene

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.adaptive.currentWindowAdaptiveInfo
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.scene.Scene
import androidx.navigation3.scene.SceneStrategy
import androidx.navigation3.scene.SceneStrategyScope
import androidx.window.core.layout.WindowSizeClass
import androidx.window.core.layout.WindowSizeClass.Companion.WIDTH_DP_EXPANDED_LOWER_BOUND
import net.mullvad.mullvadvpn.core.animation.ENTER_TRANSITION_SLIDE_FACTOR
import net.mullvad.mullvadvpn.core.animation.TRANSITION_DEFAULT_DURATION_MS

/** A [Scene] that displays a list and a detail [NavEntry] side-by-side in a 40/60 split. */
class ListDetailScene<T : Any>(
    override val key: Any,
    override val previousEntries: List<NavEntry<T>>,
    val listEntry: NavEntry<T>,
    val detailEntry: NavEntry<T>,
) : Scene<T> {
    override val entries: List<NavEntry<T>> = listOf(listEntry, detailEntry)
    override val content: @Composable (() -> Unit) = {
        Row(modifier = Modifier.fillMaxSize()) {
            Column(modifier = Modifier.weight(0.4f)) {
                listEntry.ContentForRole(ListDetailSceneStrategy.Role.List)
            }

            Column(modifier = Modifier.weight(0.6f)) {
                AnimatedContent(
                    targetState = detailEntry,
                    contentKey = { entry -> entry.contentKey },
                    transitionSpec = {
                        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) +
                            slideIntoContainer(
                                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                                towards = AnimatedContentTransitionScope.SlideDirection.Start,
                                initialOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() },
                            ) togetherWith
                            slideOutOfContainer(
                                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                                towards = AnimatedContentTransitionScope.SlideDirection.End,
                                targetOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() },
                            ) + fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
                    },
                ) { entry ->
                    entry.ContentForRole(ListDetailSceneStrategy.Role.Detail)
                }
            }
        }
    }
}

@Composable
private fun <T : Any> NavEntry<T>.ContentForRole(role: SceneRole) {
    CompositionLocalProvider(LocalSceneRole provides role) { Content() }
}

/**
 * This `CompositionLocal` can be used by a `NavEntry` to determine what role it is playing in the
 * current scene.
 */
val LocalSceneRole = compositionLocalOf<SceneRole> { SceneRole.Unknown }

interface SceneRole {
    data object Unknown : SceneRole
}

@Composable
fun <T : Any> rememberListDetailSceneStrategy(): ListDetailSceneStrategy<T> {
    val windowSizeClass = currentWindowAdaptiveInfo().windowSizeClass

    return remember(windowSizeClass) { ListDetailSceneStrategy(windowSizeClass) }
}

/**
 * A [SceneStrategy] that returns a [ListDetailScene] if:
 * - the window width is over 600dp
 * - A `Detail` entry is the last item in the back stack
 * - A `List` entry is in the back stack
 *
 * Notably, when the detail entry changes the scene's key does not change. This allows the scene,
 * rather than the NavDisplay, to handle animations when the detail entry changes.
 */
class ListDetailSceneStrategy<T : Any>(val windowSizeClass: WindowSizeClass) : SceneStrategy<T> {

    sealed interface Role : SceneRole {
        data object List : Role

        data object Detail : Role
    }

    @Suppress("ReturnCount")
    override fun SceneStrategyScope<T>.calculateScene(entries: List<NavEntry<T>>): Scene<T>? {

        if (!isListDetailTargetWidth()) {
            return null
        }

        val detailEntry =
            entries.lastOrNull()?.takeIf { it.metadata[ROLE_KEY] is Role.Detail } ?: return null
        val listEntry = entries.findLast { it.metadata[ROLE_KEY] is Role.List } ?: return null

        // We use the list's contentKey to uniquely identify the scene.
        // This allows the detail panes to be animated in and out by the scene, rather than
        // having NavDisplay animate the whole scene out when the selected detail item changes.
        val sceneKey = listEntry.contentKey

        return ListDetailScene(
            key = sceneKey,
            previousEntries = entries.dropLast(1),
            listEntry = listEntry,
            detailEntry = detailEntry,
        )
    }

    fun isListDetailTargetWidth(): Boolean =
        windowSizeClass.isWidthAtLeastBreakpoint(WIDTH_DP_EXPANDED_LOWER_BOUND)

    companion object {
        internal const val ROLE_KEY = "ListDetailScene-Role"

        /**
         * Helper function to add metadata to a [NavEntry] indicating it can be displayed in the
         * list pane of a [ListDetailScene].
         */
        fun listPane() = mapOf(ROLE_KEY to Role.List)

        /**
         * Helper function to add metadata to a [NavEntry] indicating it can be displayed in the
         * detail pane of a the [ListDetailScene].
         */
        fun detailPane() = mapOf(ROLE_KEY to Role.Detail)
    }
}
