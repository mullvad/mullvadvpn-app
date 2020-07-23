package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.TimeLeftFormatter
import org.joda.time.DateTime
import org.joda.time.Duration

class RemainingTimeLabel(val context: Context, val view: View) {
    private val resources = context.resources
    private val formatter = TimeLeftFormatter(resources)

    private val expiredColor = context.getColor(R.color.red)
    private val normalColor = context.getColor(R.color.white60)

    private val label = view.findViewById<TextView>(R.id.label)

    var accountExpiry: DateTime? = null
        set(value) {
            field = value
            updateLabel()
        }

    private fun updateLabel() {
        val expiry = accountExpiry

        if (expiry != null) {
            val remainingTime = Duration(DateTime.now(), expiry)

            if (remainingTime.isShorterThan(Duration.ZERO)) {
                label.setText(R.string.out_of_time)
                label.setTextColor(expiredColor)
            } else {
                label.setText(formatter.format(expiry, remainingTime))
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
