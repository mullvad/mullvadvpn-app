package net.mullvad.mullvadvpn.repository

import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import org.junit.jupiter.api.Test

@ExperimentalCoroutinesApi
class ChangelogRepositoryTest {

    private val mockDataProvider: IChangelogDataProvider = mockk()

    private val changelogRepository =
        ChangelogRepository(
            mockDataProvider,
            mockk(relaxed = true),
            mockk(),
            UnconfinedTestDispatcher(),
        )

    @Test
    fun `when given a changelog text should return a list of correctly formatted strings`() {
        // Arrange
        val testChangelog =
            "- Added very nice new feature with a very long descriptive message-with-hyphens\n" +
                "  - about how it works...\n" +
                "- Fixed super bad leak."
        val expectedResult =
            listOf(
                "Added very nice new feature with a very long descriptive message-with-hyphens\n  - about how it works...",
                "Fixed super bad leak.",
            )
        every { mockDataProvider.getChangelog() } returns testChangelog

        // Act
        val result = changelogRepository.getLastVersionChanges()

        // Assert
        assertLists(expectedResult, result)
    }
}
