package net.mullvad.mullvadvpn.lib.common.util

inline fun <T, reified E : T> List<T>.getFirstInstanceOrNull(): E? =
    this.filterIsInstance<E>().firstOrNull()
