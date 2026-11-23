package net.mullvad.mullvadvpn.feature.home.impl.welcome

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.ui.tag.GET_STARTED_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class WelcomeScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun contentState(
        tunnelState: TunnelState = TunnelState.Disconnected(),
        accountNumber: AccountNumber? = null,
    ): Lc<Unit, WelcomeUiState> =
        WelcomeUiState(
                tunnelState = tunnelState,
                accountNumber = accountNumber,
                deviceName = null,
            )
            .toLc()

    private fun ComposeContext.initScreen(
        state: Lc<Unit, WelcomeUiState> = Lc.Loading(Unit),
        onSettingsClick: () -> Unit = {},
        onAccountClick: () -> Unit = {},
        onDisconnectClick: () -> Unit = {},
        onGetStartedClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            WelcomeScreen(
                state = state,
                onSettingsClick = onSettingsClick,
                onAccountClick = onAccountClick,
                onDisconnectClick = onDisconnectClick,
                onGetStartedClick = onGetStartedClick,
            )
        }
    }

    @Test
    fun testOnGetStartedClick() = composeExtension.use {
        // Arrange
        val clickHandler: () -> Unit = mockk(relaxed = true)
        initScreen(state = contentState(), onGetStartedClick = clickHandler)

        // Act
        onNodeWithTag(GET_STARTED_BUTTON_TEST_TAG).performClick()

        // Assert
        verify(exactly = 1) { clickHandler() }
    }

    @Test
    fun testDefaultState() = composeExtension.use {
        // Arrange
        initScreen()

        // Assert
        onNodeWithText("New account created").assertExists()
    }

    @Test
    fun testShowAccountNumber() = composeExtension.use {
        // Arrange
        val rawAccountNumber = AccountNumber("1111222233334444")
        val expectedAccountNumber = "1111 2222 3333 4444"
        initScreen(state = contentState(accountNumber = rawAccountNumber))

        // Assert
        onNodeWithText(expectedAccountNumber).assertExists()
    }

    @Test
    fun testOnDisconnectClick() = composeExtension.use {
        // Arrange
        val clickHandler: () -> Unit = mockk(relaxed = true)
        val tunnelState: TunnelState = mockk(relaxed = true)
        every { tunnelState.isSecured() } returns true
        initScreen(
            state = contentState(tunnelState = tunnelState),
            onDisconnectClick = clickHandler,
        )

        // Act
        onNodeWithText("Disconnect").performClick()

        // Assert
        verify { clickHandler() }
    }
}
