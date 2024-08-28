package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation

const val ACCOUNT_NUMBER_SEPARATOR = " "
const val ACCOUNT_NUMBER_CHUNK_SIZE = 4

fun accountNumberVisualTransformation() = VisualTransformation {
    val transformedString =
        it.chunked(ACCOUNT_NUMBER_CHUNK_SIZE).joinToString(ACCOUNT_NUMBER_SEPARATOR)
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
