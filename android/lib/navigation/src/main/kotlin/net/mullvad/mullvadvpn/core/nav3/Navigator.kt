package net.mullvad.mullvadvpn.core.nav3

// import android.util.Log
import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.NavKey

/**
 * Handles navigation events (forward and back) by updating the navigation state.
 *
 * @param state - The navigation state that will be updated in response to navigation events.
 */
class Navigator(val state: NavigationState) {

    /**
     * Navigate to a navigation key
     *
     * @param key - the navigation key to navigate to.
     */
    fun navigate(key: NavKey, clearBackStack: Boolean = false) {
        state.backStack.apply {
            if (clearBackStack) {
                clear()
            }
            if (key != state.backStack.lastOrNull()) {
                add(key)
            }
        }
        //        Log.d("Navigator", "after push: " + state.backStack.toList().map {
        // it.javaClass.simpleName })
    }

    /** Go back to the previous navigation key. */
    fun goBack() {
        if (state.backStack.size > 1) {
            state.backStack.removeLastOrNull()
        }
        //        Log.d("Navigator", "after pop: " + state.backStack.toList().map {
        // it.javaClass.simpleName} )
    }

    inline fun <reified T : NavigationResult> goBack(resultStore: ResultStore, result: T) {
        resultStore.setResult(result)
        goBack()
    }

    /** Go back to the previous navigation key. */
    @SuppressLint("ComposableNaming")
    @Composable
    inline fun <reified T : NavigationResult> goBack(result: T) {
        LocalResultStore.current.setResult(result)
        goBack()
    }
}
