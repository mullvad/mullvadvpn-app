package net.mullvad.mullvadvpn.lib.model

import arrow.core.nonEmptyListOf
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.model.HighlightedString.Companion.findHighlights
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNull
import org.junit.jupiter.api.assertThrows

class HighlightedStringTest {
    @Test
    fun `find highlights when query is not present should not match`() {
        // Arrange
        val text = "Text"

        // Act
        val result = findHighlights(text = text, query = "xyz")

        // Assert
        assertNull(result)
    }

    @Test
    fun `find highlights when query is present once should match once`() {
        // Arrange
        val text = "Text with delimiter"

        // Act
        val result = findHighlights(text = text, query = "with")

        // Assert
        assertEquals(HighlightedString(nonEmptyListOf(5..8), text), result)
    }

    @Test
    fun `find highlights when query is present twice should match twice`() {
        // Arrange
        val text = "Text with multiple delimiters"

        // Act
        val result = findHighlights(text = text, "it")

        // Assert
        assertEquals(HighlightedString(nonEmptyListOf(6..7, 24..25), text), result)
    }

    @Test
    fun `find highlights when query has two words and is present should match`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = findHighlights(text = text, query = "two one")

        // Assert
        assertEquals(HighlightedString(nonEmptyListOf(4..10), text), result)
    }

    @Test
    fun `find highlights when matching query with multiple words should not match if not all words are present`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = findHighlights(text = text, query = "two four")

        // Assert
        assertNull(result)
    }

    @Test
    fun `should throw if highlights are out of bounds`() {
        // Arrange
        val text = "Short text"

        // Act & Assert
        assertThrows<IllegalArgumentException> {
            HighlightedString(nonEmptyListOf(0..4, 10..15), text)
        }
    }
}
