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
        val rightPadding = resources.getDimensionPixelSize(R.dimen.cell_inner_spacing)
        val verticalPadding = resources.getDimensionPixelSize(R.dimen.cell_label_vertical_padding)

        layoutParams = LayoutParams(0, LayoutParams.WRAP_CONTENT, 1.0f)
        setPadding(0, verticalPadding, rightPadding, verticalPadding)

        setTextColor(context.getColor(R.color.white))
        setTextSize(TypedValue.COMPLEX_UNIT_PX, resources.getDimension(R.dimen.text_medium_plus))
        setTypeface(null, Typeface.BOLD)
    }

    protected var footer: TextView? = null
        set(value) {
            field = value?.apply {
                val horizontalPadding =
                    resources.getDimensionPixelSize(R.dimen.cell_footer_horizontal_padding)
                val topPadding = resources.getDimensionPixelSize(R.dimen.cell_footer_top_padding)

                layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
                setPadding(horizontalPadding, topPadding, horizontalPadding, 0)

                setTextColor(context.getColor(R.color.white60))
                setTextSize(TypedValue.COMPLEX_UNIT_PX, resources.getDimension(R.dimen.text_small))
            }
        }

    protected var cell: LinearLayout = this
        set(value) {
            field = value.apply {
                val height = resources.getDimensionPixelSize(R.dimen.cell_height)
                val leftPadding = resources.getDimensionPixelSize(R.dimen.cell_left_padding)
                val rightPadding = resources.getDimensionPixelSize(R.dimen.cell_right_padding)

                setFocusable(true)
                isClickable = true
                gravity = Gravity.CENTER
                orientation = HORIZONTAL
                minimumHeight = height

                setBackgroundResource(R.drawable.cell_button_background)
                setPadding(leftPadding, 0, rightPadding, 0)

                addView(label)

                setOnClickListener { onClickListener?.invoke() }
            }
        }

    var onClickListener: (() -> Unit)? = null

    @JvmOverloads
    constructor(
        context: Context,
        attributes: AttributeSet? = null,
        defaultStyleAttribute: Int = 0,
        defaultStyleResource: Int = 0,
        footer: TextView? = null
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
        this.footer = footer
        loadAttributes(attributes)
    }

    private fun loadAttributes(attributes: AttributeSet?) {
        context.theme.obtainStyledAttributes(attributes, R.styleable.TextAttribute, 0, 0).apply {
            try {
                label.text = getString(R.styleable.TextAttribute_text) ?: ""
            } finally {
                recycle()
            }
        }

        context.theme.obtainStyledAttributes(attributes, R.styleable.Cell, 0, 0).apply {
            try {
                getString(R.styleable.Cell_footer)?.let { footerText ->
                    if (footer == null) {
                        footer = TextView(context)
                    }

                    footer?.text = footerText
                }
            } finally {
                recycle()
            }
        }

        setUp()
    }

    private fun setUp() {
        if (footer != null) {
            cell = LinearLayout(context).apply {
                layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
            }

            isClickable = false
            orientation = VERTICAL

            addView(cell)
            addView(footer)
        } else {
            cell = this
        }
    }
}
