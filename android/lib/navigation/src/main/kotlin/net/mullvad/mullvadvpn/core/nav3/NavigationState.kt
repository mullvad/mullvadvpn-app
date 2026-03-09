package net.mullvad.mullvadvpn.core.nav3

import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.runtime.toMutableStateList
import androidx.lifecycle.viewmodel.navigation3.rememberViewModelStoreNavEntryDecorator
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.rememberDecoratedNavEntries
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator

/** Create a navigation state that persists config changes and process death. */
@Composable
fun rememberNavigationState(startKey: NavKey): NavigationState {
    val backStack = rememberNavBackStack(startKey)

    return remember(backStack) { NavigationState(backStack = backStack) }
}

/**
 * State holder for navigation state.
 *
 * @param backStack - the navigation back stack.
 */
class NavigationState(val backStack: NavBackStack<NavKey>) {
    val currentKey: NavKey by derivedStateOf { backStack.last() }
}

/** Convert NavigationState into NavEntries. */
@Composable
fun NavigationState.toEntries(
    entryProvider: (NavKey) -> NavEntry<NavKey>
): SnapshotStateList<NavEntry<NavKey>> =
    rememberDecoratedNavEntries(
            entryDecorators =
                listOf(
                    // This the default decorator that must always be present.
                    rememberSaveableStateHolderNavEntryDecorator(),
                    // Clear the viewmodel when the NavEntry from which the ViewModel was created
                    // is popped of the backstack.
                    rememberViewModelStoreNavEntryDecorator(),
                ),
            backStack = backStack,
            entryProvider = entryProvider,
        )
        .toMutableStateList()
