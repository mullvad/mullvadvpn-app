package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.dialog.ChangelogDialog
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.ChangeLog
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ChangelogDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    @MockK lateinit var mockedViewModel: ChangelogViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testShowChangeLogWhenNeeded() {
        // Arrange
        every { mockedViewModel.markChangeLogAsRead() } just Runs

        composeTestRule.setContentWithTheme {
            ChangelogDialog(
                ChangeLog(
                    changes = listOf(CHANGELOG_ITEM),
                    version = CHANGELOG_VERSION,
                ),
                onDismiss = { mockedViewModel.markChangeLogAsRead() }
            )
        }

        // Check changelog content showed within dialog
        composeTestRule.onNodeWithText(CHANGELOG_ITEM).assertExists()

        // perform click on Got It button to check if dismiss occur
        composeTestRule.onNodeWithText(CHANGELOG_BUTTON_TEXT).performClick()

        // Assert
        verify { mockedViewModel.markChangeLogAsRead() }
    }

    companion object {
        private const val CHANGELOG_BUTTON_TEXT = "Got it!"
        private const val CHANGELOG_ITEM = "Changelog item"
        private const val CHANGELOG_VERSION = "1234.5"
    }
}
