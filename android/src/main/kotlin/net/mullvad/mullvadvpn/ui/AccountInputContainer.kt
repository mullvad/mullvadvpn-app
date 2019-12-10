package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.util.AttributeSet
import android.widget.LinearLayout
import net.mullvad.mullvadvpn.R

class AccountInputContainer : LinearLayout {
    enum class BorderState {
        UNFOCUSED,
        FOCUSED,
        ERROR
    }

    private val errorBorder = resources.getDrawable(R.drawable.account_input_border_error, null)
    private val focusedBorder = resources.getDrawable(R.drawable.account_input_border_focused, null)

    var borderState = BorderState.UNFOCUSED
        set(value) {
            field = value

            overlay.clear()

            when (value) {
                BorderState.UNFOCUSED -> {}
                BorderState.FOCUSED -> overlay.add(focusedBorder)
                BorderState.ERROR -> overlay.add(errorBorder)
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
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
    }

    protected override fun onLayout(
        changed: Boolean,
        left: Int,
        top: Int,
        right: Int,
        bottom: Int
    ) {
        super.onLayout(changed, left, top, right, bottom)

        if (changed) {
            val width = right - left
            val height = bottom - top

            errorBorder.setBounds(0, 0, width, height)
            focusedBorder.setBounds(0, 0, width, height)
        }
    }
}
