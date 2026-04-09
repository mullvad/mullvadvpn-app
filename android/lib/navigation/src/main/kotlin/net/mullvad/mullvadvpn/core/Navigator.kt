package net.mullvad.mullvadvpn.core

import androidx.compose.runtime.toMutableStateList

/** Handles navigation events (forward and back) by updating the navigation state. */
class Navigator(
    private val state: NavigationState,
    val resultStore: ResultStore,
    val screenIsListDetailTargetWidth: Boolean,
) {

    /** A view of the previous back stack as it was before the last navigation/pop event. */
    var previousBackStack: List<NavKey2> = state.backStack.toList()
        private set

    val backStack: List<NavKey2> by state::backStack

    /**
     * Navigate to a navigation key.
     *
     * @param keys the navigation keys to navigate to.
     * @param clearBackStack if true clears the back stack before pushing the new key
     */
    fun navigate(vararg keys: NavKey2, clearBackStack: Boolean = false) {
        previousBackStack = state.backStack.toList()

        state.backStack.apply {
            if (clearBackStack) {
                clear()
            }

            keys.forEach { key ->
                if (key != state.backStack.lastOrNull()) {
                    add(key)
                }
            }
        }
    }

    /**
     * Navigate to a navigation key and pop the topmost entry of the stack before pushing the new
     * key.
     *
     * @param key the navigation key to navigate to.
     */
    fun navigateReplaceTop(key: NavKey2) {
        previousBackStack = state.backStack.toList()

        state.backStack.apply {
            state.backStack.removeLastOrNull()

            if (key != state.backStack.lastOrNull()) {
                add(key)
            }
        }
    }

    /** Go back to the previous navigation key. If there is no previous key, do nothing. */
    fun goBack() {
        val backStackBeforePop = state.backStack.toList()
        if (tryPop()) previousBackStack = backStackBeforePop
    }

    /**
     * Go back to the previous navigation key with a result. If there is no previous key, do
     * nothing.
     */
    inline fun <reified T : NavResult> goBack(result: T) {
        resultStore.setResult(result)
        goBack()
    }

    /**
     * Attempts to pop the back stack back to a specific destination.
     *
     * @param key The topmost destination to retain
     * @param inclusive Whether the given destination should also be popped.
     * @return true if the stack was popped at least once and the user has been navigated to another
     *   destination, false otherwise
     */
    fun goBackUntil(key: NavKey2, inclusive: Boolean = false): Boolean {
        val backStackBeforePop = state.backStack.toList()

        val index = state.backStack.indexOfLast { it.javaClass == key.javaClass }
        if (index == -1) return false

        // coerceAtLeast(1) guarantees we can't end up with an empty backstack
        val keepUntil = (if (inclusive) index else index + 1).coerceAtLeast(1)
        state.backStack.removeRange(keepUntil, state.backStack.size)

        val didPop = state.backStack.size != backStackBeforePop.size
        if (didPop) {
            previousBackStack = backStackBeforePop
        }
        return didPop
    }

    private fun tryPop(): Boolean =
        if (state.backStack.size > 1) {
            state.backStack.removeLastOrNull()
            true
        } else {
            false
        }
}

// Used for previews
val EmptyNavigator =
    Navigator(
        state = NavigationState(emptyList<NavKey2>().toMutableStateList()),
        resultStore = ResultStore(),
        screenIsListDetailTargetWidth = false,
    )
