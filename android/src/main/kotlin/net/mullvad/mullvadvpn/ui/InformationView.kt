package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.util.AttributeSet
import android.util.TypedValue
import android.view.LayoutInflater
import android.widget.LinearLayout
import android.widget.TextView
import net.mullvad.mullvadvpn.R

open class InformationView : LinearLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.information_view, this).apply {
                setOnClickListener { onClick?.invoke() }
                setEnabled(false)
            }
        }

    private val description: TextView = findViewById(R.id.description)
    private val informationDisplay: TextView = findViewById(R.id.information_display)

    var information: String
        get() = informationDisplay.text?.toString() ?: ""
        set(value) {
            informationDisplay.text = value
            setEnabled(value != null)
        }

    var onClick: (() -> Unit)? = null

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
        val backgroundResource = TypedValue()

        context.theme.resolveAttribute(
            android.R.attr.selectableItemBackground,
            backgroundResource,
            true
        )

        orientation = VERTICAL
        setBackgroundResource(backgroundResource.resourceId)
    }

    private fun loadAttributes(attributes: AttributeSet) {
        val styleableId = R.styleable.InformationView

        context.theme.obtainStyledAttributes(attributes, styleableId, 0, 0).apply {
            try {
                description.text = getString(R.styleable.InformationView_description) ?: ""
            } finally {
                recycle()
            }
        }
    }
}
