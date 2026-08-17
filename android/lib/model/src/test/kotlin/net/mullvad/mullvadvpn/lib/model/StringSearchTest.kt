package net.mullvad.mullvadvpn.lib.model

import kotlin.test.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNotNull
import org.junit.jupiter.api.assertNull

class StringSearchTest {
    @Test
    fun `string search when query is not present should not match`() {
        // Arrange
        val text = "Text"

        // Act
        val result = text.search("xyz")

        // Assert
        assertNull(result)
    }

    @Test
    fun `string search when query is present once should match once`() {
        // Arrange
        val text = "Text with delimiter"

        // Act
        val result = text.search("with")

        // Assert
        assertNotNull(result)
        assertEquals(1, result.matchRange.size)
        assertEquals(5..8, result.matchRange[0])
    }

    @Test
    fun `string search when query is present twice should match first occurrence`() {
        // Arrange
        val text = "Text with multiple delimiters"

        // Act
        val result = text.search("it")

        // Assert
        assertNotNull(result)
        assertEquals(1, result.matchRange.size)
        assertEquals(6..7, result.matchRange[0])
    }

    @Test
    fun `string search when query has two words and is present should match`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = text.search("two one")

        // Assert
        assertNotNull(result)
        assertEquals(1, result.matchRange.size)
        assertEquals(4..10, result.matchRange[0])
    }

    @Test
    fun `string search when matching query with multiple words should not match if not all words are present`() {
        // Arrange
        val text = "one two one three one"

        // Act
        val result = text.search("two four")

        // Assert
        assertNull(result)
    }

    @Test
    fun `search string when fuzzy match should return all character highlights`() {
        // Arrange
        val text = "a1b2c3"

        // Act
        val result = text.search("abc")

        // Assert
        assertNotNull(result)
        assertEquals(3, result.matchRange.size)
        assertEquals(0..0, result.matchRange[0])
        assertEquals(2..2, result.matchRange[1])
        assertEquals(4..4, result.matchRange[2])
    }

    @Test
    fun `search string with multiple matches should return the expected score order`() {
        // Arrange
        val texts = listOf("Switch Access", "Messages", "Sanity Check", "Personal Safety")

        val result = texts.mapNotNull { it.search("sa") }.sortedByDescending { it }.map { it.text }

        // Matched order is:
        // 1. start of first word
        // 2. start of second word
        // 3. substring match
        // 4. fuzzy match
        assertEquals(listOf("Sanity Check", "Personal Safety", "Messages", "Switch Access"), result)
    }
}
