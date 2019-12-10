package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.TextView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AccountCache
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.PeriodType

class RemainingTimeLabel(val context: Context, val accountCache: AccountCache, val view: View) {
    private val resources = context.resources

    private val expiredColor = resources.getColor(R.color.red)
    private val normalColor = resources.getColor(R.color.white60)

    private val label = view.findViewById<TextView>(R.id.remaining_time)

    private var updateJob: Job? = null

    fun onResume() {
        accountCache.apply {
            refetch()

            onAccountDataChange = { _, accountExpiry ->
                updateJob?.cancel()
                updateJob = updateLabel(accountExpiry)
            }
        }
    }

    fun onPause() {
        accountCache.onAccountDataChange = null
        updateJob?.cancel()
    }

    private fun updateLabel(expiry: DateTime?) = GlobalScope.launch(Dispatchers.Main) {
        if (expiry != null) {
            val remainingTime = Duration(DateTime.now(), expiry)

            if (remainingTime.isShorterThan(Duration.ZERO)) {
                label.setText(R.string.out_of_time)
                label.setTextColor(expiredColor)
            } else {
                val remainingTimeInfo =
                    remainingTime.toPeriodTo(expiry, PeriodType.yearMonthDayTime())

                if (remainingTimeInfo.years > 0) {
                    label.setText(getRemainingText(R.plurals.years_left, remainingTimeInfo.years))
                } else if (remainingTimeInfo.months > 0) {
                    label.setText(getRemainingText(R.plurals.months_left, remainingTimeInfo.months))
                } else if (remainingTimeInfo.days > 0) {
                    label.setText(getRemainingText(R.plurals.days_left, remainingTimeInfo.days))
                } else {
                    label.setText(R.string.less_than_a_day_left)
                }

                label.setTextColor(normalColor)
            }
        } else {
            label.text = ""
        }
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
