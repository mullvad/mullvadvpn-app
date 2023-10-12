package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import java.lang.Integer.min

const val VOUCHER_SEPARATOR = "-"
const val VOUCHER_CHUNK_SIZE = 4
const val MAX_VOUCHER_LENGTH = 16

fun vouchersVisualTransformation() = VisualTransformation { text ->
    var out =
        text
            .substring(0, min(MAX_VOUCHER_LENGTH, text.length))
            .chunked(VOUCHER_CHUNK_SIZE)
            .joinToString(VOUCHER_SEPARATOR)
    if (
        text.length % VOUCHER_CHUNK_SIZE == 0 &&
            text.isNotEmpty() &&
            text.length < MAX_VOUCHER_LENGTH
    ) {
        out += VOUCHER_SEPARATOR
    }
    TransformedText(
        AnnotatedString(out),
        object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                val res = offset + offset / ACCOUNT_TOKEN_CHUNK_SIZE
                // Limit max input to 19 characters (16 voucher - 3 dividers)
                return min(
                    res,
                    MAX_VOUCHER_LENGTH + MAX_VOUCHER_LENGTH / ACCOUNT_TOKEN_CHUNK_SIZE - 1
                )
            }

            override fun transformedToOriginal(offset: Int): Int =
                offset - offset / (ACCOUNT_TOKEN_CHUNK_SIZE + 1)
        }
    )
}
