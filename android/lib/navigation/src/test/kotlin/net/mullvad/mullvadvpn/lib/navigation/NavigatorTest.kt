package net.mullvad.mullvadvpn.lib.navigation

import androidx.compose.runtime.toMutableStateList
import kotlin.test.assertEquals
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavigationState
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.ResultStore
import org.junit.jupiter.api.Test

@Parcelize object Key1 : NavKey2

@Parcelize object Key2 : NavKey2

@Parcelize object Key3 : NavKey2

@Parcelize object Key4 : NavKey2

class NavigatorTestTest {

    @Test
    fun `ensure navigating pushes a nav key to the backstack correctly`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3))

        assertEquals(listOf(Key1, Key2, Key3), navigator.previousBackStack)
        assertEquals(listOf(Key1, Key2, Key3), navigator.backStack.toList())

        navigator.navigate(Key4)

        assertEquals(listOf(Key1, Key2, Key3), navigator.previousBackStack)
        assertEquals(listOf(Key1, Key2, Key3, Key4), navigator.backStack.toList())
    }

    @Test
    fun `ensure navigating while replacing the top works`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3))

        navigator.navigateReplaceTop(Key4)

        assertEquals(listOf(Key1, Key2, Key3), navigator.previousBackStack)
        assertEquals(listOf(Key1, Key2, Key4), navigator.backStack.toList())
    }

    @Test
    fun `ensure navigating while clearing the backstack works`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3))

        navigator.navigate(Key4, clearBackStack = true)

        assertEquals(listOf(Key1, Key2, Key3), navigator.previousBackStack)
        assertEquals(listOf(Key4), navigator.backStack.toList())
    }

    @Test
    fun `ensure goBack pops the backstack`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3))

        navigator.goBack()

        assertEquals(listOf(Key1, Key2, Key3), navigator.previousBackStack)
        assertEquals(listOf(Key1, Key2), navigator.backStack.toList())
    }

    @Test
    fun `ensure goBack doesn't pop the last item off the backstack`() {
        val navigator = createNavigator(listOf(Key1))

        navigator.goBack()

        assertEquals(listOf(Key1), navigator.previousBackStack)
        assertEquals(listOf(Key1), navigator.backStack.toList())
    }

    @Test
    fun `ensure goBackUntil pops the backstack to the topmost instance of the given key`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3, Key2, Key4))

        navigator.goBackUntil(Key2)
        assertEquals(listOf(Key1, Key2, Key3, Key2), navigator.backStack.toList())
    }

    @Test
    fun `ensure goBackUntil inclusive pops the backstack to key before the topmost instance of the given key`() {
        val navigator = createNavigator(listOf(Key1, Key2, Key3, Key2, Key4))

        navigator.goBackUntil(Key2, inclusive = true)
        assertEquals(listOf(Key1, Key2, Key3), navigator.backStack.toList())
    }

    @Test
    fun `ensure the previous backstack is not changed if no pop was done`() {
        val navigator = createNavigator(listOf(Key1))
        navigator.navigate(Key2)
        navigator.goBack()

        assertEquals(listOf(Key1), navigator.backStack.toList())
        assertEquals(listOf(Key1, Key2), navigator.previousBackStack)

        navigator.goBack()
        assertEquals(listOf(Key1), navigator.backStack.toList())
        assertEquals(listOf(Key1, Key2), navigator.previousBackStack)

        navigator.goBackUntil(Key1, inclusive = true)
        assertEquals(listOf(Key1), navigator.backStack.toList())
        assertEquals(listOf(Key1, Key2), navigator.previousBackStack)
    }

    private fun createNavigator(backStack: List<NavKey2>): Navigator {
        return Navigator(
            state = NavigationState(backStack.toMutableStateList()),
            resultStore = ResultStore(),
            screenIsListDetailTargetWidth = false,
        )
    }
}
