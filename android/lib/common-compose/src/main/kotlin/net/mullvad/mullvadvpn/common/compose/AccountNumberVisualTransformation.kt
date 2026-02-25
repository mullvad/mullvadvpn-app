package net.mullvad.mullvadvpn.common.compose

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
