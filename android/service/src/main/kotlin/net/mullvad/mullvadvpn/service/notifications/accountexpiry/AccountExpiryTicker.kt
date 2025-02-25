package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import java.time.Duration
import java.time.Instant
import java.time.ZonedDateTime
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import net.mullvad.mullvadvpn.lib.common.util.millisFromNow

sealed interface AccountExpiryTicker {
    data object NotWithinThreshold : AccountExpiryTicker

    data class Tick(val expiresIn: Duration) : AccountExpiryTicker

    companion object {
        fun tickerFlow(
            expiry: ZonedDateTime,
            tickStart: Duration,
            updateInterval: (expiry: ZonedDateTime) -> Duration,
        ): Flow<AccountExpiryTicker> = flow {
            expiry.millisFromNow().let { expiryMillis ->
                if (expiryMillis <= 0) {
                    // Has expired.
                    emit(Tick(Duration.ZERO))
                    return@flow
                }
                if (expiryMillis > tickStart.toMillis()) {
                    // Emit NotWithinThreshold if no expiry notification should be provided.
                    emit(NotWithinThreshold)
                    // Delay until the time we should start emitting.
                    delay(expiryMillis - tickStart.toMillis() + 1)
                }
            }

            var currentUpdateInterval = updateInterval(expiry).toMillis()

            do {
                emit(Tick(Duration.between(Instant.now(), expiry)))
                delay(millisUntilNextUpdate(expiry.millisFromNow(), currentUpdateInterval))
                currentUpdateInterval = updateInterval(expiry).toMillis()
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
