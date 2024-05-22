package net.mullvad.mullvadvpn.lib.common.util

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withTimeoutOrNull

suspend fun <T> Flow<T>.firstOrNullWithTimeout(timeMillis: Long): T? {
    return withTimeoutOrNull(timeMillis) { firstOrNull() }
}
