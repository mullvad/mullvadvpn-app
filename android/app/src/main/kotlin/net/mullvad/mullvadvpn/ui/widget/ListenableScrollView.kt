package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.widget.ScrollView
import net.mullvad.mullvadvpn.util.ListenableScrollableView

class ListenableScrollView : ScrollView, ListenableScrollableView {
    override val horizontalScrollOffset
        get() = scrollX
    override val verticalScrollOffset
        get() = scrollY

    override var onScrollListener: ((Int, Int, Int, Int) -> Unit)? = null

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute)

    override fun onScrollChanged(left: Int, top: Int, oldLeft: Int, oldTop: Int) {
        super.onScrollChanged(left, top, oldLeft, oldTop)
        onScrollListener?.invoke(left, top, oldLeft, oldTop)
    }
}
