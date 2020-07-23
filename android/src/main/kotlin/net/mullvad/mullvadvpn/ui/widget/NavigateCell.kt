package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.widget.ImageView
import net.mullvad.mullvadvpn.R

open class NavigateCell : Cell {
    private val chevron = ImageView(context).apply {
        val width = resources.getDimensionPixelSize(R.dimen.chevron_width)
        val height = resources.getDimensionPixelSize(R.dimen.chevron_height)

        layoutParams = LayoutParams(width, height, 0.0f)
        alpha = 0.6f

        setImageResource(R.drawable.icon_chevron)
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
        cell.addView(chevron)
    }
}
