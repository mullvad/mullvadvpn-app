package net.mullvad.mullvadvpn.lib.model

import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.model.HighlightedString.Companion.findHighlights
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class HighlightedStringTest {
    @Test
    fun `partial match when ignore case is false should not match on different case`() {
        // Arrange
        val text = "Text with CASE"

        // Act
        val result = findHighlights(text = text, query = "WITH", ignoreCase = false)

        // Assert
        assertEquals(HighlightedString(emptyList(), "Text with CASE"), result)
    }

    @Test
    fun `partial match when query is not present should not match`() {
        // Arrange
        val text = "Text"

        // Act
        val result = findHighlights(text = text, query = "xyz", ignoreCase = false)

        // Assert
        assertEquals(HighlightedString(emptyList(), text), result)
    }

    @Test
    fun `partial match when query is present once should match once`() {
        // Arrange
        val text = "Text with delimiter"

        // Act
        val result = findHighlights(text = text, query = "with", ignoreCase = false)

        // Assert
        assertEquals(HighlightedString(5..8, text), result)
    }

    @Test
    fun `partial match when query is present twice should match twice`() {
        // Arrange
        val text = "Text with multiple delimiters"

        // Act
        val result = findHighlights(text = text, "it", ignoreCase = false)

        // Assert
        assertEquals(HighlightedString(listOf(6..7, 24..25), text), result)
    }

    @Test
    fun `partial match when ignore case is true should match regardless of case`() {
        // Arrange
        val text = "Text ignore case"

        // Act
        val result = findHighlights(text = text, query = "TEXT", ignoreCase = true)

        // Assert
        assertEquals(HighlightedString(listOf(0..3), text), result)
    }

    @Test
    fun `partial match when limit is zero should match on all occurrences`() {
        // Arrange
        val text = "one two one two one"

        // Act
        val result = findHighlights(text = text, query = "two", ignoreCase = false, limit = 0)

        // Assert
        assertEquals(HighlightedString(listOf(4..6, 12..14), text), result)
    }

    @Test
    fun `partial match when limit is 1 should only match once`() {
        // Arrange
        val text = "one two one two one"

        // Act
        val result = findHighlights(text = text, query = "two", ignoreCase = false, limit = 1)

        // Assert
        assertEquals(HighlightedString(listOf(4..6), text), result)
    }

    @Test
    fun `partial match when matching multiple substrings should split on all`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = findHighlights(text = text, query = "two three", ignoreCase = false, limit = 0)

        // Assert
        assertEquals(HighlightedString(listOf(4..6, 12..16), text), result)
    }

    @Test
    fun `partial match when matching query with multiple words should only match the correct words`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = findHighlights(text = text, query = "two four", ignoreCase = false, limit = 0)

        // Assert
        assertEquals(HighlightedString(listOf(4..6), text), result)
    }

    @Test
    fun `partial match when limit is 2 and query contains multiple words should only match on first two occurrences`() {
        // Arrange
        val text = "one two one two three one"

        // Act
        val result = findHighlights(text = text, query = "two three", ignoreCase = false, limit = 2)

        // Assert
        assertEquals(HighlightedString(listOf(4..6, 12..14), text), result)
    }

    @Test()
    fun `should throw if highlights are out of bounds`() {
        // Arrange
        val text = "Short text"

        // Act & Assert
        assertThrows<IllegalArgumentException> { HighlightedString(listOf(0..4, 10..15), text) }
    }
}
