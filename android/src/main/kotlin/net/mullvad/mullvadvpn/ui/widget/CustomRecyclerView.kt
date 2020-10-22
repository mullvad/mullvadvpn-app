package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.support.v7.widget.RecyclerView
import android.util.AttributeSet
import net.mullvad.mullvadvpn.util.ListenableScrollableView

class CustomRecyclerView : RecyclerView, ListenableScrollableView {
    override var horizontalScrollOffset = 0
    override var verticalScrollOffset = 0

    override var onScrollListener: ((Int, Int, Int, Int) -> Unit)? = null

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {
        }

    override fun onScrolled(horizontalDelta: Int, verticalDelta: Int) {
        super.onScrolled(horizontalDelta, verticalDelta)

        dispatchScrollEvent(horizontalDelta, verticalDelta)
    }

    private fun dispatchScrollEvent(horizontalDelta: Int, verticalDelta: Int) {
        val oldHorizontalScrollOffset = horizontalScrollOffset
        val oldVerticalScrollOffset = verticalScrollOffset

        horizontalScrollOffset += horizontalDelta
        verticalScrollOffset += verticalDelta

        onScrollListener?.invoke(
            horizontalScrollOffset,
            verticalScrollOffset,
            oldHorizontalScrollOffset,
            oldVerticalScrollOffset
        )
    }
}
