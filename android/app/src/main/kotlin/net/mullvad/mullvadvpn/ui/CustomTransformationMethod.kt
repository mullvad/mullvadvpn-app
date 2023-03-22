package net.mullvad.mullvadvpn.ui

import android.graphics.Rect
import android.text.method.PasswordTransformationMethod
import android.text.method.TransformationMethod
import android.view.View

private const val BIG_DOT_CHAR = '●'
private const val DOT_CHAR = '\u2022'
private const val EMPTY_STRING = ""
private const val SPACE_CHAR = ' '

class GroupedTransformationMethod() : TransformationMethod {
    override fun getTransformation(source: CharSequence?, view: View?): CharSequence {
        return source?.groupWithSpaces() ?: EMPTY_STRING
    }

    override fun onFocusChanged(
        view: View?,
        sourceText: CharSequence?,
        focused: Boolean,
        direction: Int,
        previouslyFocusedRect: Rect?
    ) {
        // No focus handling required.
    }
}

class GroupedPasswordTransformationMethod() : PasswordTransformationMethod() {
    override fun getTransformation(source: CharSequence?, view: View?): CharSequence {
        return if (source != null && view != null) {
            super.getTransformation(source, view)
                ?.toString()
                ?.replace(DOT_CHAR, BIG_DOT_CHAR)
                ?.groupWithSpaces()
                ?: EMPTY_STRING
        } else {
            EMPTY_STRING
        }
    }
}

private fun CharSequence.groupWithSpaces(groupCharSize: Int = 4): CharSequence {
    return fold(StringBuilder()) { formattedText, nextDigit ->
        if ((formattedText.length % (groupCharSize + 1)) == groupCharSize) {
            formattedText.append(SPACE_CHAR)
        }
        formattedText.append(nextDigit)
    }
}
