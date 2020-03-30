package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

class TimeSinceLabel(val context: Context, val view: View) {
    private val resources = context.resources
    private val label = view.findViewById<TextView>(R.id.time_since)

    private val periodType = PeriodType.standard()
        .withMillisRemoved()
        .withSecondsRemoved()

    var timeInstant: DateTime? = null
        set(value) {
            field = value
            updateLabel()
        }

    var visibility
        get() = label.visibility
        set(value) {
            label.visibility = value
        }

    private fun updateLabel() {
        val instant = timeInstant

        if (instant != null) {
            val elapsedTime = Duration(instant, DateTime.now())
            val elapsedTimeInfo = elapsedTime.toPeriodTo(instant, periodType)

            if (elapsedTimeInfo.years > 0) {
                label.setText(getRemainingText(R.plurals.years_ago, elapsedTimeInfo.years))
            } else if (elapsedTimeInfo.months > 0) {
                label.setText(getRemainingText(R.plurals.months_ago, elapsedTimeInfo.months))
            } else if (elapsedTimeInfo.days > 0) {
                label.setText(getRemainingText(R.plurals.days_ago, elapsedTimeInfo.days))
            } else if (elapsedTimeInfo.hours > 0) {
                label.setText(getRemainingText(R.plurals.hours_ago, elapsedTimeInfo.hours))
            } else if (elapsedTimeInfo.minutes > 0) {
                label.setText(getRemainingText(R.plurals.minutes_ago, elapsedTimeInfo.minutes))
            } else {
                label.setText(R.string.less_than_a_minute_ago)
            }
        } else {
            label.text = ""
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
