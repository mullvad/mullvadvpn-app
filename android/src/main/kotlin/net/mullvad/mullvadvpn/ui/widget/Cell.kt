package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.graphics.Typeface
import android.util.AttributeSet
import android.util.TypedValue
import android.view.Gravity
import android.widget.LinearLayout
import android.widget.TextView
import net.mullvad.mullvadvpn.R

open class Cell : LinearLayout {
    private val label = TextView(context).apply {
        val horizontalPadding =
            resources.getDimensionPixelSize(R.dimen.cell_label_horizontal_padding)
        val verticalPadding = resources.getDimensionPixelSize(R.dimen.cell_label_vertical_padding)

        layoutParams = LayoutParams(0, LayoutParams.WRAP_CONTENT, 1.0f)
        setPadding(horizontalPadding, verticalPadding, horizontalPadding, verticalPadding)

        setTextColor(context.getColor(R.color.white))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 20.0f)
        setTypeface(null, Typeface.BOLD)
    }

    var onClickListener: (() -> Unit)? = null

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
        isClickable = true
        gravity = Gravity.CENTER
        orientation = HORIZONTAL

        setBackgroundResource(R.drawable.cell_button_background)

        resources.getDimensionPixelSize(R.dimen.cell_horizontal_padding).let { padding ->
            setPadding(padding, 0, padding, 0)
        }

        addView(label)

        setOnClickListener { onClickListener?.invoke() }
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
}
