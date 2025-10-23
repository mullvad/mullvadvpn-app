package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import java.time.Duration
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import org.junit.jupiter.api.Test

class AccountExpiryNotificationTriggerTest {

    @Test
    fun `long account expiry should trigger 3 days before expiry`() {
        val now = ZonedDateTime.now()

        val threeMonthsExpiry = now.plusHours(90 * 24)
        val trigger1 = accountExpiryNotificationTriggerAt(now, threeMonthsExpiry)
        assertEquals(87, Duration.between(now, trigger1).toDays())

        val fourAndHalfDaysExpiry = now.plusHours(4 * 24 + 12)
        val trigger2 = accountExpiryNotificationTriggerAt(now, fourAndHalfDaysExpiry)
        assertEquals(Duration.ofDays(1).plusHours(12), Duration.between(now, trigger2))
    }

    @Test
    fun `account expiry that more than 2 days but less than 3 days should trigger 2 days before expiry`() {
        val now = ZonedDateTime.now()
        val expiry = now.plusHours(50)
        val trigger = accountExpiryNotificationTriggerAt(now, expiry)
        // Because acc
        assertEquals(2, Duration.between(now, trigger).toHours())
    }

    @Test
    fun `account expiry that is more than 1 day but less than 2 days should trigger 1 day before expiry`() {
        val now = ZonedDateTime.now()
        val expiry = now.plusHours(36).plusMinutes(20).plusSeconds(7)
        val trigger = accountExpiryNotificationTriggerAt(now, expiry)
        assertEquals(
            Duration.ofHours(12).plusMinutes(20).plusSeconds(7),
            Duration.between(now, trigger),
        )
    }

    @Test
    fun `account expiry that is less than 24 hours should trigger when account expires`() {
        val now = ZonedDateTime.now()
        val expiry = now.plusHours(2).plusMinutes(1).plusSeconds(30)
        val trigger = accountExpiryNotificationTriggerAt(now, expiry)

        assertEquals(
            Duration.ofHours(2).plusMinutes(1).plusSeconds(30),
            Duration.between(now, trigger),
        )
    }

    @Test
    fun `account that expires now should return now`() {
        val now = ZonedDateTime.now()
        val trigger = accountExpiryNotificationTriggerAt(now, now)

        assertEquals(Duration.ofMillis(0), Duration.between(now, trigger))
    }

    @Test
    fun `account expiry that is in the past should return the account expiry date`() {
        val now = ZonedDateTime.now()
        val expiry = now.minusDays(1).minusHours(17).minusMinutes(3).minusSeconds(40)
        val trigger = accountExpiryNotificationTriggerAt(now, expiry)

        assertEquals(expiry, trigger)
    }
}
