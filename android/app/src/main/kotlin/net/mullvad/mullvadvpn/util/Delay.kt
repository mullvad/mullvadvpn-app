package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay

suspend fun <T> delayAtLeast(duration: Long, f: suspend () -> T): T = coroutineScope {
    val result = async { f() }
    delay(timeMillis = duration)
    result.await()
}
