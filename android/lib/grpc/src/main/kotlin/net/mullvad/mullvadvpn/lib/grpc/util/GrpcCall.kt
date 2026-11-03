package net.mullvad.mullvadvpn.lib.grpc.util

import co.touchlab.kermit.Logger
import com.squareup.wire.GrpcCall
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay

private val CONNECT_RETRY_DELAY = 100.milliseconds
private val EXPONENTIAL_BACK_OFF_INCREASE = 20.milliseconds
private val MAX_RETRY_DELAY = 3.seconds

// This will retry indefinitely until the connection does not throw a
// LocalSocketConnectException. This is needed because the daemon may not have set up the socket
// file yet leading to errors. The retrying stops if the calling coroutine is canceled.
// The delay between attempts starts at CONNECT_RETRY_DELAY and grows exponentially (the increase
// doubles for every attempt) up to MAX_RETRY_DELAY, so that the first attempts are very frequent.
internal suspend fun <S : Any, R : Any> GrpcCall<S, R>.executeWithRetry(request: S): R {
    var retryDelay = CONNECT_RETRY_DELAY
    var increase = EXPONENTIAL_BACK_OFF_INCREASE
    while (true) {
        try {
            return clone().execute(request)
        } catch (e: LocalSocketConnectException) {
            Logger.w("Failed to connect to daemon", e)
            delay(retryDelay)
            retryDelay = minOf(retryDelay + increase, MAX_RETRY_DELAY)
            increase *= 2
        }
    }
}
