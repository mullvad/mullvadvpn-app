package net.mullvad.mullvadvpn.lib.common.util

import kotlin.time.Duration
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.withIndex
import kotlinx.coroutines.withTimeoutOrNull

suspend fun <T> Flow<T>.firstOrNullWithTimeout(timeMillis: Long): T? {
    return withTimeoutOrNull(timeMillis) { firstOrNull() }
}

@OptIn(FlowPreview::class)
fun <T> Flow<T>.debounceFirst(firstTimeout: Duration, otherTimeout: Duration): Flow<T> =
    withIndex()
        .debounce {
            if (it.index == 0) {
                firstTimeout
            } else {
                otherTimeout
            }
        }
        .map { it.value }
