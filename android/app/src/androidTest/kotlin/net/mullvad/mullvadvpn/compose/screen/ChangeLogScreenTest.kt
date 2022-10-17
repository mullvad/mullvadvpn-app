package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.component.AppTheme
import net.mullvad.mullvadvpn.repository.ChangeLogState
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ChangeLogScreenTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @MockK
    lateinit var mockedViewModel: AppChangesViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        every { mockedViewModel.getChangesList() } returns ArrayList()
        every { mockedViewModel.shouldShowChanges() } returns true
        every { mockedViewModel.setDialogShowed() } just Runs
    }

    @Test
    fun testShowChangeLogWhenNeeded() {
        // Arrange
        every {
            mockedViewModel.changeLogState
        } returns MutableStateFlow(ChangeLogState.ShouldShow)

        // Act
        composeTestRule.setContent {
            AppTheme {
                ChangesListScreen(mockedViewModel) {}
            }
        }

        // Assert
        composeTestRule
            .onNodeWithText(CHANGELOG_SUBTITLE)
            .assertExists()
    }

    companion object {
        private const val CHANGELOG_SUBTITLE = "Changes in this version:"
    }
}
