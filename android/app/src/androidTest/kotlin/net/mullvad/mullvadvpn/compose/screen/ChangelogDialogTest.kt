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
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.component.AppTheme
import net.mullvad.mullvadvpn.compose.component.ChangelogDialog
import net.mullvad.mullvadvpn.viewmodel.ChangelogDialogUiState
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ChangelogDialogTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @MockK
    lateinit var mockedViewModel: ChangelogViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testShowChangeLogWhenNeeded() {
        // Arrange
        every {
            mockedViewModel.changelogDialogUiState
        } returns MutableStateFlow(ChangelogDialogUiState.Show(listOf(CHANGELOG_ITEM)))
        every {
            mockedViewModel.dismissChangelogDialog()
        } just Runs

        composeTestRule.setContent {
            AppTheme {
                ChangelogDialog(
                    changesList = listOf(CHANGELOG_ITEM),
                    version = CHANGELOG_VERSION,
                    onDismiss = {
                        mockedViewModel.dismissChangelogDialog()
                    }
                )
            }
        }

        // Check changelog content showed within dialog
        composeTestRule
            .onNodeWithText(CHANGELOG_ITEM)
            .assertExists()

        // perform click on Got It button to check if dismiss occur
        composeTestRule
            .onNodeWithText(CHANGELOG_BUTTON_TEXT)
            .performClick()

        // Assert
        verify { mockedViewModel.dismissChangelogDialog() }
    }

    companion object {
        private const val CHANGELOG_BUTTON_TEXT = "Got it!"
        private const val CHANGELOG_ITEM = "Changelog item"
        private const val CHANGELOG_VERSION = "1234.5"
    }
}
