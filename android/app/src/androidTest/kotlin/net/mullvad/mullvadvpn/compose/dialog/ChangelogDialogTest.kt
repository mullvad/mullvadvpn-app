package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.impl.annotations.MockK
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.AppInfoViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogUiState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ChangelogDialogTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @MockK lateinit var mockedViewModel: AppInfoViewModel

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initDialog(state: ChangelogUiState, onDismiss: () -> Unit = {}) {
        setContentWithTheme { ChangelogDialog(state = state, onDismiss = onDismiss) }
    }

    @Test
    fun testShowChangeLogWhenNeeded() =
        composeExtension.use {
            // Arrange
            initDialog(
                state =
                    ChangelogUiState(changes = listOf(CHANGELOG_ITEM), version = CHANGELOG_VERSION),
                onDismiss = {},
            )

            // Check changelog content showed within dialog
            onNodeWithText(CHANGELOG_ITEM).assertExists()

            // perform click on Got It button to check if dismiss occur
            onNodeWithText(CHANGELOG_BUTTON_TEXT).performClick()
        }

    companion object {
        private const val CHANGELOG_BUTTON_TEXT = "Got it!"
        private const val CHANGELOG_ITEM = "Changelog item"
        private const val CHANGELOG_VERSION = "1234.5"
    }
}
