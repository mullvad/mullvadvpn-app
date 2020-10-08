package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.util.TypedValue
import android.view.Gravity
import android.view.LayoutInflater
import android.widget.LinearLayout
import android.widget.TextView
import net.mullvad.mullvadvpn.R

class BackButton : LinearLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.settings_back_button, this)
        }

    private val label = container.findViewById<TextView>(R.id.label)

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {
        loadAttributes(attributes)
    }

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {
            loadAttributes(attributes)
        }

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
        loadAttributes(attributes)
    }

    init {
        setFocusable(true)
        isClickable = true
        gravity = Gravity.CENTER_VERTICAL or Gravity.LEFT
        orientation = HORIZONTAL

        resources.getDimensionPixelSize(R.dimen.settings_back_button_padding).let { padding ->
            setPadding(padding, padding, padding, padding)
        }

        loadBackground()
    }

    private fun loadAttributes(attributes: AttributeSet) {
        context.theme.obtainStyledAttributes(attributes, R.styleable.TextAttribute, 0, 0).apply {
            try {
                label.text = getString(R.styleable.TextAttribute_text) ?: ""
            } finally {
                recycle()
            }
        }
    }

    private fun loadBackground() {
        val typedValue = TypedValue()

        context.theme.resolveAttribute(android.R.attr.selectableItemBackground, typedValue, true)

        setBackgroundResource(typedValue.resourceId)
    }
}
