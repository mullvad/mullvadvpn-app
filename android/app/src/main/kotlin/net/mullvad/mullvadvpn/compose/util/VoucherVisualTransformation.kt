package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation

const val VOUCHER_SEPARATOR = "-"
const val VOUCHER_CHUNK_SIZE = 4

fun vouchersVisualTransformation() = VisualTransformation { text ->
    val out = text.chunked(VOUCHER_CHUNK_SIZE).joinToString(VOUCHER_SEPARATOR)
    TransformedText(
        AnnotatedString(out),
        object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int =
                offset + (offset - 1) / ACCOUNT_TOKEN_CHUNK_SIZE

            override fun transformedToOriginal(offset: Int): Int =
                offset - (offset - 1) / (ACCOUNT_TOKEN_CHUNK_SIZE + 1)
        }
    )
}
