package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.content.Intent
import android.graphics.Typeface
import android.net.Uri
import android.util.AttributeSet
import android.util.TypedValue
import android.view.Gravity
import android.widget.ImageView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class AppVersionCell : Cell {
    private val warningIcon = ImageView(context).apply {
        layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)

        resources.getDimensionPixelSize(R.dimen.cell_inner_spacing).let { padding ->
            setPadding(0, 0, padding, 0)
        }

        setImageResource(R.drawable.icon_alert)
    }

    private val versionLabel = TextView(context).apply {
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

    private val externalLinkIcon = ImageView(context).apply {
        layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)
        alpha = 0.6f

        setImageResource(R.drawable.icon_extlink)
    }

    var updateAvailable by observable(false) { _, _, updateAvailable ->
        if (updateAvailable) {
            warningIcon.visibility = VISIBLE
            footer?.visibility = VISIBLE
        } else {
            warningIcon.visibility = GONE
            footer?.visibility = GONE
        }
    }

    var version by observable("") { _, _, version ->
        versionLabel.text = version
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
        cell.addView(warningIcon, 0)
        cell.addView(versionLabel)
        cell.addView(externalLinkIcon)

        onClickListener = { openLink() }
    }

    private fun openLink() {
        val url = context.getString(R.string.download_url)
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))

        context.startActivity(intent)
    }
}
