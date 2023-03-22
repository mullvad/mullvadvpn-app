package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.graphics.Typeface
import android.net.Uri
import android.util.AttributeSet
import android.util.TypedValue
import android.view.Gravity
import android.widget.ImageView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class AppVersionCell : UrlCell {
    private val warningIcon =
        ImageView(context).apply {
            val iconSize = resources.getDimensionPixelSize(R.dimen.app_version_warning_icon_size)

            layoutParams = LayoutParams(iconSize, iconSize, 0.0f)

            resources.getDimensionPixelSize(R.dimen.cell_inner_spacing).let { padding ->
                setPadding(0, 0, padding, 0)
            }

            setImageResource(R.drawable.icon_alert)
        }

    private val versionLabel =
        TextView(context).apply {
            layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)
            gravity = Gravity.RIGHT

            resources.getDimensionPixelSize(R.dimen.cell_inner_spacing).let { padding ->
                setPadding(padding, 0, padding, 0)
            }

            setTextColor(context.getColor(R.color.white60))
            setTextSize(TypedValue.COMPLEX_UNIT_PX, resources.getDimension(R.dimen.text_small))
            setTypeface(null, Typeface.BOLD)

            text = ""
        }

    var updateAvailable by
        observable(false) { _, _, updateAvailable ->
            if (updateAvailable) {
                warningIcon.visibility = VISIBLE
                footer?.visibility = VISIBLE
            } else {
                warningIcon.visibility = GONE
                footer?.visibility = GONE
            }
        }

    var version by observable("") { _, _, version -> versionLabel.text = version }

    @JvmOverloads
    constructor(
        context: Context,
        attributes: AttributeSet? = null,
        defaultStyleAttribute: Int = 0,
        defaultStyleResource: Int = 0
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource)

    init {
        cell.addView(warningIcon, 0)
        cell.addView(versionLabel, cell.getChildCount() - 1)

        if (url == null) {
            url = Uri.parse(context.getString(R.string.download_url))
        }
    }
}
