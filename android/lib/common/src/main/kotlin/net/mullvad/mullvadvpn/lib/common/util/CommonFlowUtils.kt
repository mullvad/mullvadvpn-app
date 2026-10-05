package net.mullvad.mullvadvpn.lib.common.util

import kotlin.time.Duration
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withTimeoutOrNull

suspend fun <T> Flow<T>.firstOrNullWithTimeout(timeout: Duration): T? {
    return withTimeoutOrNull(timeout) { firstOrNull() }
}
