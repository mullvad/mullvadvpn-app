package net.mullvad.mullvadvpn.lib.common.test

import kotlin.coroutines.CoroutineContext
import kotlin.test.assertTrue
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest

// Cancels the given scope before runTest drains the scheduler. This prevents while(true) polling
// loops indefinitely running and causing the test to hang.
fun runAndCancelContextTest(
    coroutineContext: CoroutineContext,
    block: suspend TestScope.() -> Unit,
) = runTest {
    try {
        block()
    } finally {
        coroutineContext.cancel()
    }
}

fun <T> assertLists(expected: List<T>, actual: List<T>, message: String? = null) =
    assertTrue(
        expected.size == actual.size &&
            expected.containsAll(actual) &&
            actual.containsAll(expected),
        message
            ?: """Expected list should have same size and contains same items.
        | Expected(${expected.size}): $expected
        | Actual(${actual.size})  : $actual
    """
                .trimMargin(),
    )

fun <T> assertSet(expected: Set<T>, actual: Set<T>, message: String? = null) =
    assertTrue(
        expected.size == actual.size &&
            expected.containsAll(actual) &&
            actual.containsAll(expected),
        message
            ?: """Expected list should have same size and contains same items.
        | Expected(${expected.size}): $expected
        | Actual(${actual.size})  : $actual
    """
                .trimMargin(),
    )
