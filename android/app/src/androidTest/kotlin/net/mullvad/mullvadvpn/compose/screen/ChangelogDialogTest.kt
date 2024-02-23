package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.dialog.ChangelogDialog
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.Changelog
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ChangelogDialogTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @MockK lateinit var mockedViewModel: ChangelogViewModel

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testShowChangeLogWhenNeeded() =
        composeExtension.use {
            // Arrange
            // Arrange
            every { mockedViewModel.markChangelogAsRead() } just Runs

            setContentWithTheme {
                ChangelogDialog(
                    Changelog(
                        changes = listOf(CHANGELOG_ITEM),
                        version = CHANGELOG_VERSION,
                    ),
                    onDismiss = { mockedViewModel.markChangelogAsRead() }
                )
            }

            // Check changelog content showed within dialog
            onNodeWithText(CHANGELOG_ITEM).assertExists()

            // perform click on Got It button to check if dismiss occur
            onNodeWithText(CHANGELOG_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedViewModel.markChangelogAsRead() }
        }

    companion object {
        private const val CHANGELOG_BUTTON_TEXT = "Got it!"
        private const val CHANGELOG_ITEM = "Changelog item"
        private const val CHANGELOG_VERSION = "1234.5"
    }
}
