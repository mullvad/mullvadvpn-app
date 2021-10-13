package net.mullvad.mullvadvpn.util

import android.text.Editable
import android.text.TextWatcher
import android.widget.EditText

class SegmentedInputFormatter(val input: EditText, var separator: Char) : TextWatcher {
    private var editing = false
    private var removing = false
    private var separatorSkipCount = 5

    var allCaps = false
    var isValidInputCharacter: (Char) -> Boolean = { _ -> true }

    var segmentSize = 4
        set(value) {
            field = value
            separatorSkipCount = value + 1
        }

    init {
        input.addTextChangedListener(this)
    }

    override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {
        if (!editing) {
            editing = true
            removing = after < count
        }
    }

    override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

    override fun afterTextChanged(text: Editable) {
        val string = text.toString()

        if (isValidInput(string)) {
            editing = false
            maybeUpdateSelection(text)
        } else {
            formatInput(text)
        }
    }

    private fun maybeUpdateSelection(text: Editable) {
        if (removing) {
            var start = input.selectionStart
            var end = input.selectionEnd
            var changed = false

            if (start % separatorSkipCount == 0 && start > 0) {
                start -= 1
                changed = true
            }

            if (end % separatorSkipCount == 0 && end > 0) {
                end -= 1
                changed = true
            }

            if (changed) {
                input.setSelection(start, end)

                if (start == end && end == text.length - 1) {
                    // The cursor was previously at the last character, and now after the character
                    // was removed it has been moved to before the separator. It's best now to
                    // remove the unnecessary trailing separator
                    text.delete(text.length - 1, text.length)
                }
            }
        }
    }

    private fun isValidInput(string: String): Boolean {
        return string
            .asSequence()
            .withIndex()
            .all { item ->
                val index = item.index
                val character = item.value

                if ((index + 1) % separatorSkipCount == 0) {
                    character == separator
                } else {
                    isValidInputCharacter(character)
                }
            }
    }

    private fun formatInput(input: Editable) {
        var index = 0
        val length = input.length
        var changed = false

        while (index < length && !changed) {
            val segmentStart = index
            val segmentEnd = index + segmentSize - 1
            val separatorPosition = segmentEnd + 1

            changed = formatSegment(input, segmentStart..segmentEnd) ||
                formatSeparator(input, separatorPosition)

            index = separatorPosition + 1
        }
    }

    private fun formatSegment(input: Editable, range: IntRange): Boolean {
        val length = input.length
        val start = range.start
        var end = range.endInclusive

        if (start < length) {
            end = minOf(end, length - 1)

            for (index in start..end) {
                val character = input[index]

                if (allCaps && character >= 'a' && character <= 'z') {
                    input.replace(index, index + 1, character.toString().toUpperCase())
                } else if (!isValidInputCharacter(character)) {
                    input.delete(index, index + 1)
                } else {
                    // Only continue looping if no changes were made to the string
                    continue
                }

                // Abort loop because the input was edited and `afterTextChanged` will be called
                // again
                return true
            }
        }

        return false
    }

    private fun formatSeparator(input: Editable, index: Int): Boolean {
        if (index < input.length && input[index] != separator) {
            input.insert(index, "$separator")
            return true
        } else {
            return false
        }
    }
}
