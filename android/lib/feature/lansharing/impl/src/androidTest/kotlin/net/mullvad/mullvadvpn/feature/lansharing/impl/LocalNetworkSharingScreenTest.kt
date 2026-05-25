package net.mullvad.mullvadvpn.feature.lansharing.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
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
class LocalNetworkSharingScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun createDefaultUiState(
        isModal: Boolean = false,
        isLanSharingEnabled: Boolean = false,
    ) = LocalNetworkSharingUiState(isModal = isModal, lanSharingEnabled = isLanSharingEnabled)

    private fun ComposeContext.initScreen(
        state: Lc<Boolean, LocalNetworkSharingUiState> = createDefaultUiState().toLc(),
        onLocalNetworkSharingEnable: (Boolean) -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            LocalNetworkSharingScreen(
                state = state,
                onBackClick = onBackClick,
                onLocalNetworkSharingEnable = onLocalNetworkSharingEnable,
            )
        }
    }

    @Test
    fun testDefaultState() = composeExtension.use {
        // Arrange
        initScreen()

        // Assert
        onNodeWithText("Local network sharing").assertExists()
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() = composeExtension.use {
        // Arrange
        initScreen(state = Lc.Loading(true))

        // Assert
        onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG).assertExists()
    }
}
