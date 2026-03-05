package net.mullvad.mullvadvpn.common.compose

import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.insert
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import kotlin.math.max

const val ACCOUNT_NUMBER_SEPARATOR = " "
const val ACCOUNT_NUMBER_CHUNK_SIZE = 4
const val PASSWORD_UNICODE = '●'

fun accountNumberVisualTransformation(showAccount: Boolean = true, showLastX: Int = 0) =
    VisualTransformation {
        val replacementLength = max(0, it.length - showLastX)
        val inputString =
            if (showAccount) it
            else {
                it.replaceRange(
                    0..<replacementLength,
                    PASSWORD_UNICODE.toString().repeat(replacementLength),
                )
            }
        val transformedString =
            inputString.chunked(ACCOUNT_NUMBER_CHUNK_SIZE).joinToString(ACCOUNT_NUMBER_SEPARATOR)
        val transformedAnnotatedString = AnnotatedString(transformedString)

        TransformedText(
            transformedAnnotatedString,
            object : OffsetMapping {
                override fun originalToTransformed(offset: Int): Int =
                    offset + (offset - 1) / ACCOUNT_NUMBER_CHUNK_SIZE

                override fun transformedToOriginal(offset: Int): Int =
                    offset - (offset - 1) / (ACCOUNT_NUMBER_CHUNK_SIZE + 1)
            },
        )
    }

fun accountNumberOutputTransformation(showAccount: Boolean = true, showLastX: Int = 0) =
    OutputTransformation {
        val originalLength = length
        val replacementLength = max(0, originalLength - showLastX)

        // Replace characters with password dots if not showing account
        if (!showAccount && replacementLength > 0) {
            replace(0, replacementLength, PASSWORD_UNICODE.toString().repeat(replacementLength))
        }

        // Insert separators between chunks (from right to left to maintain correct positions)
        // Start from the last chunk boundary that is within the string (not at the end)
        var position = ((length - 1) / ACCOUNT_NUMBER_CHUNK_SIZE) * ACCOUNT_NUMBER_CHUNK_SIZE
        while (position > 0) {
            insert(position, ACCOUNT_NUMBER_SEPARATOR)
            position -= ACCOUNT_NUMBER_CHUNK_SIZE
        }
    }
