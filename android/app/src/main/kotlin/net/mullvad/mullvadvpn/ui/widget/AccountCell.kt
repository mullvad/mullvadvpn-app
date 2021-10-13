package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.graphics.Typeface
import android.util.AttributeSet
import android.util.TypedValue
import android.view.Gravity
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.TimeLeftFormatter
import org.joda.time.DateTime
import org.joda.time.Duration

class AccountCell : NavigateCell {
    private val formatter = TimeLeftFormatter(resources)

    private val expiredColor = context.getColor(R.color.red)
    private val normalColor = context.getColor(R.color.white60)

    private val remainingTimeLabel = TextView(context).apply {
        layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)
        gravity = Gravity.RIGHT

        resources.getDimensionPixelSize(R.dimen.cell_inner_spacing).let { padding ->
            setPadding(padding, 0, padding, 0)
        }

        setAllCaps(true)
        setTextColor(normalColor)
        setTextSize(TypedValue.COMPLEX_UNIT_PX, resources.getDimension(R.dimen.text_small))
        setTypeface(null, Typeface.BOLD)

        text = ""
    }

    var accountExpiry by observable<DateTime?>(null) { _, _, expiry ->
        remainingTimeLabel.apply {
            if (expiry != null) {
                val remainingTime = Duration(DateTime.now(), expiry)

                if (remainingTime.isShorterThan(Duration.ZERO)) {
                    setText(R.string.out_of_time)
                    setTextColor(expiredColor)
                } else {
                    setText(formatter.format(expiry, remainingTime))
                    setTextColor(normalColor)
                }
            } else {
                text = ""
            }
        }
    }

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {}

    init {
        cell.addView(remainingTimeLabel, cell.childCount - 1)
    }

    private fun getRemainingText(pluralId: Int, quantity: Int): String {
        return resources.getQuantityString(pluralId, quantity, quantity)
    }
}
