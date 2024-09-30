package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.Period

fun expiryTickerFlow(
    expiry: DateTime,
    tickStart: Duration,
    updateInterval: (expiry: DateTime) -> Duration,
): Flow<Period> = flow {
    var currentUpdateInterval = updateInterval(expiry).millis

    expiry.millisFromNow().let { expiryMillis ->
        if (expiryMillis <= 0) {
            // Has expired.
            emit(Period.ZERO)
            return@flow
        } else if (expiryMillis <= currentUpdateInterval) {
            emit(Period(DateTime.now(), expiry))
            delay(expiry.millisFromNow())
            emit(Period.ZERO)
            return@flow
        } else if (expiryMillis > tickStart.millis) {
            // Delay until the time we should start emitting.
            delay(expiryMillis - tickStart.millis + 1)
        }
    }

    // Always emit at start of flow.
    emit(Period(DateTime.now(), expiry))

    // Delay until the next update interval.
    delay(expiry.millisFromNow() % currentUpdateInterval)

    var delayCount = calculateDelaysNeeded(expiry.millisFromNow(), currentUpdateInterval)

    while (delayCount > 0) {
        emit(Period(DateTime.now(), expiry))
        delay(currentUpdateInterval)

        val newUpdateInterval = updateInterval(expiry).millis
        if (newUpdateInterval != currentUpdateInterval) {
            delayCount = calculateDelaysNeeded(expiry.millisFromNow(), newUpdateInterval)
            currentUpdateInterval = newUpdateInterval
        } else {
            delayCount -= 1
        }
    }

    // We may have remaining time if the update interval wasn't a multiple of the remaining time.
    delay(expiry.millisFromNow())

    // We have now expired.
    emit(Period.ZERO)
}

// Calculate how many times we need to delay and and emit until the expiry time is reached.
// Note that the returned delays may add upp to less than the remaining time, for example
// if we have 100ms remaining and currentUpdateIntervalMillis is 40ms this function will return 2.
private fun calculateDelaysNeeded(millisUntilExpiry: Long, currentUpdateIntervalMillis: Long): Int {
    return if (millisUntilExpiry <= 0) {
        0
    } else {
        (millisUntilExpiry / currentUpdateIntervalMillis).toInt()
    }
}

private fun DateTime.millisFromNow(): Long = Duration(DateTime.now(), this).millis
