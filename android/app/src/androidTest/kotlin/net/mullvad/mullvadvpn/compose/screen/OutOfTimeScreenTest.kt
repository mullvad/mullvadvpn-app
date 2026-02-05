package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.AddTimeViewModel
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension
import org.koin.core.context.loadKoinModules
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.module

@OptIn(ExperimentalTestApi::class)
class OutOfTimeScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private val addTimeViewModel: AddTimeViewModel = mockk(relaxed = true)

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        loadKoinModules(module { viewModel { addTimeViewModel } })
        every { addTimeViewModel.uiState } returns
            MutableStateFlow<Lc<Unit, AddTimeUiState>>(Lc.Loading(Unit))
    }

    private fun ComposeContext.initScreen(
        state: OutOfTimeUiState = OutOfTimeUiState(),
        onDisconnectClick: () -> Unit = {},
        onRedeemVoucherClick: () -> Unit = {},
        onSettingsClick: () -> Unit = {},
        onAccountClick: () -> Unit = {},
        onPlayPaymentInfoClick: () -> Unit = {},
    ) {

        setContentWithTheme {
            OutOfTimeScreen(
                state = state,
                onDisconnectClick = onDisconnectClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onSettingsClick = onSettingsClick,
                onAccountClick = onAccountClick,
                onPlayPaymentInfoClick = onPlayPaymentInfoClick,
            )
        }
    }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            initScreen(state = OutOfTimeUiState(deviceName = ""))

            // Assert
            onNodeWithText(
                    "Either buy credit on our website or redeem a voucher.",
                    substring = true,
                )
                .assertDoesNotExist()
        }

    @Test
    fun testOpenAccountView() =
        composeExtension.use {
            val mockClickListener: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                onAccountClick = mockClickListener,
            )

            onNodeWithContentDescription(label = "Account").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testClickDisconnect() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    OutOfTimeUiState(
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        deviceName = "",
                        showSitePayment = true,
                    ),
                onDisconnectClick = mockClickListener,
            )

            // Act
            onNodeWithText("Disconnect").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testShowPendingPaymentInfoDialog() =
        composeExtension.use {
            // Arrange
            val mockOnPlayPaymentInfoClick: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = OutOfTimeUiState(showSitePayment = true, verificationPending = true),
                onPlayPaymentInfoClick = mockOnPlayPaymentInfoClick,
            )

            // Act
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).assertExists()

            // Assert
            verify(exactly = 1) { mockOnPlayPaymentInfoClick.invoke() }
        }

    @Test
    fun testShowVerificationInProgress() =
        composeExtension.use {
            // Arrange
            initScreen(state = OutOfTimeUiState(showSitePayment = true, verificationPending = true))

            // Assert
            onNodeWithText("Google Play payment pending").assertExists()
        }
}
