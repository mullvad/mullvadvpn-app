package net.mullvad.mullvadvpn.lib.common.util

import kotlin.test.assertEquals
import org.junit.jupiter.api.Test

class StringExtensionsTest {

    @Test
    fun `when delimiter is not present should return list with the original string`() {
        // Arrange
        val text = "Text"

        // Act
        val result = text.splitIncludingDelimiters("xyz", ignoreCase = false)

        // Assert
        assertEquals(listOf("Text"), result)
    }

    @Test
    fun `when delimiter is present once should return correct parts including delimiter`() {
        // Arrange
        val text = "Text with delimiter"

        // Act
        val result = text.splitIncludingDelimiters("with", ignoreCase = false)

        // Assert
        assertEquals(listOf("Text ", "with", " delimiter"), result)
    }

    @Test
    fun `when delimiter is present twice should return correct parts including delimiter twice`() {
        // Arrange
        val text = "Text with multiple delimiters"

        // Act
        val result = text.splitIncludingDelimiters("it", ignoreCase = false)

        // Assert
        assertEquals(listOf("Text w", "it", "h multiple delim", "it", "ers"), result)
    }

    @Test
    fun `when ignore case is true should match regardless of case`() {
        // Arrange
        val text = "Text ignore case"

        // Act
        val result = text.splitIncludingDelimiters("TEXT", ignoreCase = true)

        // Assert
        assertEquals(listOf("Text", " ignore case"), result)
    }

    @Test
    fun `when ignore case is true and delimiter match output should preserve case`() {
        // Arrange
        val text = "Text preserve case"

        // Act
        val result = text.splitIncludingDelimiters("TEXT", ignoreCase = true)

        // Assert
        assertEquals("Text", result[0])
    }

    @Test
    fun `when ignore case is false should not match if case is different`() {
        // Arrange
        val text = "Text with CASE"

        // Act
        val result = text.splitIncludingDelimiters("WITH", ignoreCase = false)

        // Assert
        assertEquals(listOf("Text with CASE"), result)
    }

    @Test
    fun `when limit is zero should split on all occurrences`() {
        // Arrange
        val text = "one two one two one"

        // Act
        val result = text.splitIncludingDelimiters("two", ignoreCase = false, limit = 0)

        // Assert
        assertEquals(listOf("one ", "two", " one ", "two", " one"), result)
    }

    @Test
    fun `when limit is one should only split on first occurrence of delimiter`() {
        // Arrange
        val text = "one two one two one"

        // Act
        val result = text.splitIncludingDelimiters("two", ignoreCase = false, limit = 1)

        // Assert
        assertEquals(listOf("one ", "two", " one two one"), result)
    }
}
