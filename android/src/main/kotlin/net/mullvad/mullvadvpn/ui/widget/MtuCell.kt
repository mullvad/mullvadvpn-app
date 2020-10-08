package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.text.Editable
import android.text.TextWatcher
import android.util.AttributeSet
import android.view.LayoutInflater
import android.widget.EditText
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

class MtuCell : Cell {
    private val input =
        (LayoutInflater.from(context).inflate(R.layout.mtu_edit_text, null) as EditText).apply {
            val width = resources.getDimensionPixelSize(R.dimen.cell_input_width)
            val height = resources.getDimensionPixelSize(R.dimen.cell_input_height)

            layoutParams = LayoutParams(width, height, 0.0f)

            addTextChangedListener(InputWatcher())
            setOnFocusChangeListener { _, newHasFocus -> hasFocus = newHasFocus }
        }

    private val validInputColor = context.getColor(R.color.white)
    private val invalidInputColor = context.getColor(R.color.red)

    var value: Int?
        get() = input.text.toString().trim().toIntOrNull()
        set(value) = input.setText(value?.toString() ?: "")

    var onSubmit: ((Int?) -> Unit)? = null

    var hasFocus by observable(false) { _, oldValue, newValue ->
        if (oldValue == true && newValue == false) {
            val mtu = value

            if (mtu == null || (mtu >= MIN_MTU_VALUE && mtu <= MAX_MTU_VALUE)) {
                onSubmit?.invoke(mtu)
            }
        }
    }

    constructor(context: Context) : super(context, TextView(context)) {}

    constructor(context: Context, attributes: AttributeSet) :
        super(context, attributes, TextView(context)) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute, TextView(context)) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(
        context,
        attributes,
        defaultStyleAttribute,
        defaultStyleResource,
        TextView(context)
    ) {}

    init {
        cell.apply {
            setEnabled(false)
            setFocusable(false)
            addView(input)
        }

        footer?.text =
            context.getString(R.string.wireguard_mtu_footer, MIN_MTU_VALUE, MAX_MTU_VALUE)
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun afterTextChanged(text: Editable) {
            val value = text.toString().trim().toIntOrNull()

            if (value != null && value >= MIN_MTU_VALUE && value <= MAX_MTU_VALUE) {
                input.setTextColor(validInputColor)
            } else {
                input.setTextColor(invalidInputColor)
            }
        }
    }
}
