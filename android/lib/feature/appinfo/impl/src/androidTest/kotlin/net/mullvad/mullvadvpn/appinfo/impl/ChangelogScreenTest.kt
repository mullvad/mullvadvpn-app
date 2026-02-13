package net.mullvad.mullvadvpn.appinfo.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.appinfo.impl.changelog.ChangelogScreen
import net.mullvad.mullvadvpn.appinfo.impl.changelog.ChangelogUiState
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ChangelogScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(state: ChangelogUiState, onBackClick: () -> Unit = {}) {
        setContentWithTheme { ChangelogScreen(state = state, onBackClick = onBackClick) }
    }

    @Test
    fun testShowChangeLogWhenNeeded() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    ChangelogUiState(changes = listOf(CHANGELOG_ITEM), version = CHANGELOG_VERSION),
                onBackClick = {},
            )

            // Check changelog version shown
            onNodeWithText(CHANGELOG_VERSION).assertExists()

            // Check changelog content showed within dialog
            onNodeWithText(CHANGELOG_ITEM).assertExists()
        }

    companion object {
        private const val CHANGELOG_ITEM = "Changelog item"
        private const val CHANGELOG_VERSION = "1234.5"
    }
}
