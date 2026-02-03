@file:Suppress("MagicNumber")

package net.mullvad.mullvadvpn.lib.common.util

import java.time.Duration
import java.time.ZonedDateTime
import kotlin.time.Duration.Companion.seconds

val ACCOUNT_EXPIRY_POLL_INTERVAL = 15.seconds

// When to start showing the account expiry in-app notification.
val ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD: Duration = Duration.ofDays(3)

// How often to update the account expiry in-app notification.
val ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL: Duration = Duration.ofDays(1)

// Calculate when the alarm that triggers the account expiry notification should be set.
fun accountExpiryNotificationTriggerAt(now: ZonedDateTime, expiry: ZonedDateTime): ZonedDateTime {
    val untilExpiry = Duration.between(now, expiry)

    return if (untilExpiry > ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD) {
        val wait = untilExpiry - ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
        now + wait
    } else {
        val wait = untilExpiry.toMillis() % ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL.toMillis()

        // If the expiry is in the past we just return it as it is.
        if (wait >= 0) now + Duration.ofMillis(wait) else expiry
    }
}
