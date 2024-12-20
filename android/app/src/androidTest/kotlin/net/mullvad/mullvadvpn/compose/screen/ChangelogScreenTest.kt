package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
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
class ChangelogScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @MockK lateinit var mockedViewModel: AppInfoViewModel

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
