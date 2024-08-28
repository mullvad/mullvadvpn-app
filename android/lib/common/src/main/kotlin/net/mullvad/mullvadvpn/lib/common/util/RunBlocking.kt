package net.mullvad.mullvadvpn.lib.common.util

import kotlin.coroutines.CoroutineContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout

fun <T> runBlockingWithTimeout(
    context: CoroutineContext,
    timeout: Long,
    block: suspend CoroutineScope.() -> T,
): T =
    runBlocking(context = context) {
        val d = async { block() }
        withTimeout(timeout) { d.await() }
    }
