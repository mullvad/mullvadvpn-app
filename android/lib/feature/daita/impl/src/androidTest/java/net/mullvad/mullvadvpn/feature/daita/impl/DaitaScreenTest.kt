package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.ui.tag.CIRCULAR_PROGRESS_INDICATOR_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class DaitaScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: Lc<Boolean, DaitaUiState> =
            DaitaUiState(daitaEnabled = false, directOnly = false).toLc(),
        onDaitaEnabled: (enable: Boolean) -> Unit = {},
        onDirectOnlyClick: (enable: Boolean) -> Unit = {},
        onDirectOnlyInfoClick: () -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            DaitaScreen(
                state,
                onDaitaEnabled,
                onDirectOnlyClick,
                onDirectOnlyInfoClick,
                onBackClick,
            )
        }
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            initScreen(state = Lc.Loading(true))

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG).assertExists()
        }
}
