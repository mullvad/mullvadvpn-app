package net.mullvad.mullvadvpn.utils

import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.util.appendTextWithStyledSubstring
import org.junit.jupiter.api.Test

class AppendTextWithStyledSubstringTest {
    @Test
    fun `empty input should result in empty output`() {

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = "",
                substring = "abc",
                substringStyle = SpanStyle(),
            )
        }

        assertEquals("", output.text)
    }

    @Test
    fun `split only should result in split only`() {

        val split = "abc"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = split,
                substring = split,
                substringStyle = SpanStyle(),
            )
        }

        assertEquals(split, output.text)
    }

    @Test
    fun `split twice should result in split twice`() {

        val split = "abcabc"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = split,
                substring = split,
                substringStyle = SpanStyle(),
            )
        }

        assertEquals(split, output.text)
    }

    @Test
    fun `split anywhere should return the input text`() {

        val text = "abca longer abc string to split abc"
        val split = "abc"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = split,
                substringStyle = SpanStyle(),
            )
        }

        assertEquals(text, output.text)
    }

    @Test
    fun `span styles should be applied to all matching substrings`() {

        val text = "Cool Cat: your username is Cool Cat."
        val split = "Cool Cat"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = split,
                substringStyle = SpanStyle(),
            )
        }

        assertEquals(text, output.text)
        assertEquals(2, output.spanStyles.size)

        assertEquals(0, output.spanStyles[0].start)
        assertEquals(8, output.spanStyles[0].end)

        assertEquals(27, output.spanStyles[1].start)
        assertEquals(35, output.spanStyles[1].end)
    }
}
