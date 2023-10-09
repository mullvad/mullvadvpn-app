package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.DurationUnit
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

sealed class AccountExpiry : Parcelable {
    @Parcelize
    data class Available(val expiryDateTime: DateTime) : AccountExpiry() {
        override fun daysLeft(): Int =
            (expiryDateTime.toInstant().millis - DateTime.now().toInstant().millis)
                .milliseconds
                .toInt(DurationUnit.DAYS)
    }

    @Parcelize data object Missing : AccountExpiry()

    fun date(): DateTime? {
        return (this as? Available)?.expiryDateTime
    }

    open fun daysLeft(): Int? {
        return (this as? Available)?.daysLeft()
    }
}
