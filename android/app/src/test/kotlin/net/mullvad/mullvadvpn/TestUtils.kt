package net.mullvad.mullvadvpn

import kotlin.test.assertTrue

fun <T> assertLists(expected: List<T>, actual: List<T>, message: String? = null) = assertTrue(
    expected.size == actual.size && expected.containsAll(actual) && actual.containsAll(expected),
    message ?: """Expected list should have same size and contains same items.
        | Expected(${expected.size}): $expected
        | Actual(${actual.size})  : $actual
    """.trimMargin()
)
