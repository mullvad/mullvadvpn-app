package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet

class ToggleCell : Cell {
    private val toggle = CellSwitch(context).apply {
        layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)
    }

    var state
        get() = toggle.state
        set(value) {
            toggle.state = value
        }

    var listener
        get() = toggle.listener
        set(value) {
            toggle.listener = value
        }

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource)

    init {
        onClickListener = { toggle() }
        cell.addView(toggle)
    }

    fun toggle() {
        toggle.toggle()
    }

    fun forcefullySetState(state: CellSwitch.State) {
        toggle.forcefullySetState(state)
    }
}
