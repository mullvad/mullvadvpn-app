@file:Suppress("MagicNumber")

package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import java.time.Duration
import kotlin.time.Duration.Companion.seconds

val ACCOUNT_EXPIRY_POLL_INTERVAL = 15.seconds
val ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL: Duration = Duration.ofDays(1)
val ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_UPDATE_INTERVAL: Duration = Duration.ofDays(1)
val ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD: Duration = Duration.ofDays(3)
