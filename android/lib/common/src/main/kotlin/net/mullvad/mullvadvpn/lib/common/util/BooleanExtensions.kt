package net.mullvad.mullvadvpn.lib.common.util

/** Returns `value` if this is true, otherwise return null. */
fun <T> Boolean.then(value: T): T? = if (this) value else null
