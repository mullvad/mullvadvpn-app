package net.mullvad.mullvadvpn.ui

import android.text.Editable
import android.text.TextWatcher
import android.widget.EditText
import net.mullvad.mullvadvpn.R

class CellInput(val input: EditText, val minValue: Int, val maxValue: Int) {
    private val resources = input.context.resources

    private val validInputColor = resources.getColor(R.color.white)
    private val invalidInputColor = resources.getColor(R.color.red)

    init {
        input.addTextChangedListener(InputWatcher())
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun afterTextChanged(text: Editable) {
            val value = text.toString().trim().toIntOrNull()

            if (value != null && value >= minValue && value <= maxValue) {
                input.setTextColor(validInputColor)
            } else {
                input.setTextColor(invalidInputColor)
            }
        }
    }
}
