package net.mullvad.mullvadvpn.lib.common.util

fun <T> Iterable<T>.indexOfFirstOrNull(predicate: (T) -> Boolean): Int? =
    indexOfFirst(predicate).takeIf { it > 0 }
