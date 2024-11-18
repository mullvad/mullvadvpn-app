package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import org.joda.time.DateTime
import org.joda.time.Duration

sealed interface AccountExpiryTicker {
    data object NotWithinThreshold : AccountExpiryTicker

    data class Tick(val expiresIn: Duration) : AccountExpiryTicker

    companion object {
        fun tickerFlow(
            expiry: DateTime,
            tickStart: Duration,
            updateInterval: (expiry: DateTime) -> Duration,
        ): Flow<AccountExpiryTicker> = flow {
            expiry.millisFromNow().let { expiryMillis ->
                if (expiryMillis <= 0) {
                    // Has expired.
                    emit(Tick(Duration.ZERO))
                    return@flow
                }
                if (expiryMillis > tickStart.millis) {
                    // Emit NotWithinThreshold if no expiry notification should be provided.
                    emit(NotWithinThreshold)
                    // Delay until the time we should start emitting.
                    delay(expiryMillis - tickStart.millis + 1)
                }
            }

            var currentUpdateInterval = updateInterval(expiry).millis

            do {
                emit(Tick(Duration(DateTime.now(), expiry)))
                delay(millisUntilNextUpdate(expiry.millisFromNow(), currentUpdateInterval))
                currentUpdateInterval = updateInterval(expiry).millis
            } while (hasAnotherEmission(expiry.millisFromNow(), currentUpdateInterval))

            // We may have remaining time if the update interval wasn't a multiple of the remaining
            // time.
            delay(expiry.millisFromNow())

            // We have now expired.
            emit(Tick(Duration.ZERO))
        }
    }
}

private fun millisUntilNextUpdate(
    millisUntilExpiry: Long,
    currentUpdateIntervalMillis: Long,
): Long =
    (millisUntilExpiry % currentUpdateIntervalMillis).let {
        if (it == 0L) currentUpdateIntervalMillis else it
    }

private fun hasAnotherEmission(millisUntilExpiry: Long, updateIntervalMillis: Long) =
    calculateDelaysNeeded(millisUntilExpiry, updateIntervalMillis) > 0

// Calculate how many times we need to delay and and emit until the expiry time is reached.
// Note that the returned delays may add upp to less than the remaining time, for example
// if we have 100ms remaining and currentUpdateIntervalMillis is 40ms this function will return 2.
private fun calculateDelaysNeeded(
    millisUntilExpiry: Long,
    currentUpdateIntervalMillis: Long,
): Long = millisUntilExpiry.coerceAtLeast(0) / currentUpdateIntervalMillis

private fun DateTime.millisFromNow(): Long = Duration(DateTime.now(), this).millis
