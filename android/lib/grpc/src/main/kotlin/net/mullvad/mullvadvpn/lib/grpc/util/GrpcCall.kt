package net.mullvad.mullvadvpn.lib.grpc.util

import co.touchlab.kermit.Logger
import com.squareup.wire.GrpcCall
import java.io.IOException
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.delay

private val CONNECT_RETRY_DELAY = 100.milliseconds

// This will retry indefinitely until the connection does not throw a
// LocalSocketConnectException. This is needed because the daemon may not have set up the socket
// file yet leading to errors. The retrying stops if the calling coroutine is canceled.
internal suspend fun <S : Any, R : Any> GrpcCall<S, R>.executeWithRetry(request: S): R {
    while (true) {
        try {
            return clone().execute(request)
        } catch (e: IOException) {
            if (e !is LocalSocketConnectException) throw e
            Logger.w("Failed to connect to daemon", e)
            delay(CONNECT_RETRY_DELAY)
        }
    }
}
