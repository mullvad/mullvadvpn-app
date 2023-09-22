package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation

const val ACCOUNT_TOKEN_SEPARATOR = " "
const val ACCOUNT_TOKEN_CHUNK_SIZE = 4

fun accountTokenVisualTransformation() = VisualTransformation {
    val transformedString =
        it.chunked(ACCOUNT_TOKEN_CHUNK_SIZE).joinToString(ACCOUNT_TOKEN_SEPARATOR)
    val transformedAnnotatedString = AnnotatedString(transformedString)

    TransformedText(
        transformedAnnotatedString,
        object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int =
                offset + (offset - 1) / ACCOUNT_TOKEN_CHUNK_SIZE

            override fun transformedToOriginal(offset: Int): Int =
                offset - (offset - 1) / (ACCOUNT_TOKEN_CHUNK_SIZE + 1)
        }
    )
}
