package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import io.mockk.every
import io.mockk.mockk
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import org.junit.jupiter.api.Test

class ChangelogRepositoryTest {

    private val mockedPreferences: SharedPreferences = mockk()
    private val mockDataProvider: IChangelogDataProvider = mockk()

    private val changelogRepository =
        ChangelogRepository(preferences = mockedPreferences, dataProvider = mockDataProvider)

    @Test
    fun `when given a changelog text should return a list of correctly formatted strings`() {
        // Arrange
        val testChangelog =
            "- Added very nice new feature with a very long descriptive message\n" +
                "  about how it works...\n" +
                "- Fixed super bad leak."
        val expectedResult =
            listOf(
                "Added very nice new feature with a very long descriptive message about how it works...",
                "Fixed super bad leak."
            )
        every { mockDataProvider.getChangelog() } returns testChangelog

        // Act
        val result = changelogRepository.getLastVersionChanges()

        // Assert
        assertLists(expectedResult, result)
    }
}
