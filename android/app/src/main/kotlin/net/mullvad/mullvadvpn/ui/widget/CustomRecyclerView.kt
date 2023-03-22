package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import androidx.recyclerview.widget.RecyclerView
import net.mullvad.mullvadvpn.util.ListenableScrollableView

class CustomRecyclerView : RecyclerView, ListenableScrollableView {
    private val customItemAnimator = CustomItemAnimator()

    override var horizontalScrollOffset = 0
    override var verticalScrollOffset = 0

    override var onScrollListener: ((Int, Int, Int, Int) -> Unit)? = null

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute)

    init {
        itemAnimator =
            customItemAnimator.apply {
                onMove = { horizontalDelta, verticalDelta ->
                    dispatchScrollEvent(horizontalDelta, verticalDelta)
                }
            }
    }

    override fun setLayoutManager(layoutManager: LayoutManager?) {
        super.setLayoutManager(layoutManager)

        customItemAnimator.layoutManager = layoutManager
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
