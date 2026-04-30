package net.mullvad.mullvadvpn.utils

import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import net.mullvad.mullvadvpn.lib.ui.component.appendTextWithStyledSubstring
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

    @Test
    fun `when substring does not match should not apply any styling`() {

        val text = "Cool Cat"
        val substring = "xyz"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = substring,
                substringStyle = SpanStyle(),
            )
        }

        assertEquals(text, output.text)
        assertTrue(output.spanStyles.isEmpty())
    }

    @Test
    fun `when substring matches part of the text in different case and ignore case is true should apply styling`() {

        val text = "Cool Cat"
        val substring = "cat"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = substring,
                substringStyle = SpanStyle(),
                ignoreCase = true,
            )
        }

        assertEquals(1, output.spanStyles.size)
    }

    @Test
    fun `when substring matches part of the text in different case and ignore case is true output text should be the same`() {

        val text = "ABCDE"
        val substring = "abc"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = substring,
                substringStyle = SpanStyle(),
                ignoreCase = true,
            )
        }

        assertEquals(text, output.text)
    }

    @Test
    fun `when substring matches part of the text in different cases and ignore case is false it should not apply styling`() {

        val text = "Cool Cat"
        val substring = "Cool cat"

        val output = buildAnnotatedString {
            appendTextWithStyledSubstring(
                text = text,
                substring = substring,
                substringStyle = SpanStyle(),
                ignoreCase = false,
            )
        }

        assertTrue(output.spanStyles.isEmpty())
    }
}
