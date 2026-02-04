package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.AddTimeViewModel
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension
import org.koin.core.context.loadKoinModules
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.module

@OptIn(ExperimentalTestApi::class)
class WelcomeScreenTest {
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
        state: Lc<Unit, WelcomeUiState> = Lc.Loading(Unit),
        onRedeemVoucherClick: () -> Unit = {},
        onSettingsClick: () -> Unit = {},
        onAccountClick: () -> Unit = {},
        onDisconnectClick: () -> Unit = {},
        navigateToDeviceInfoDialog: () -> Unit = {},
        onPlayPaymentInfoClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            WelcomeScreen(
                state = state,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onSettingsClick = onSettingsClick,
                onAccountClick = onAccountClick,
                navigateToDeviceInfoDialog = navigateToDeviceInfoDialog,
                onDisconnectClick = onDisconnectClick,
                onPlayPaymentInfoClick = onPlayPaymentInfoClick,
            )
        }
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            initScreen()

            // Assert
            onNodeWithText("Congrats!").assertExists()
            onNodeWithText("Here’s your account number. Save it!").assertExists()
        }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            initScreen()

            // Assert
            onNodeWithText(
                    "Either buy credit on our website or redeem a voucher.",
                    substring = true,
                )
                .assertDoesNotExist()
        }

    @Test
    fun testShowAccountNumber() =
        composeExtension.use {
            // Arrange
            val rawAccountNumber = AccountNumber("1111222233334444")
            val expectedAccountNumber = "1111 2222 3333 4444"
            initScreen(
                state =
                    WelcomeUiState(
                            tunnelState = TunnelState.Disconnected(),
                            accountNumber = rawAccountNumber,
                            deviceName = null,
                            showSitePayment = false,
                            verificationPending = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText(expectedAccountNumber).assertExists()
        }

    @Test
    fun testShowPendingPaymentInfoDialog() =
        composeExtension.use {
            // Arrange
            val mockShowPendingInfo = mockk<() -> Unit>(relaxed = true)
            initScreen(
                state =
                    WelcomeUiState(
                            tunnelState = TunnelState.Disconnected(),
                            accountNumber = null,
                            deviceName = null,
                            showSitePayment = false,
                            verificationPending = true,
                        )
                        .toLc(),
                onPlayPaymentInfoClick = mockShowPendingInfo,
            )

            // Act
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

            // Assert
            verify(exactly = 1) { mockShowPendingInfo() }
        }

    @Test
    fun testShowVerificationInProgress() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    WelcomeUiState(
                            tunnelState = TunnelState.Disconnected(),
                            accountNumber = null,
                            deviceName = null,
                            showSitePayment = false,
                            verificationPending = true,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("Google Play payment pending").assertExists()
        }

    @Test
    fun testOnDisconnectClick() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            val tunnelState: TunnelState = mockk(relaxed = true)
            every { tunnelState.isSecured() } returns true
            initScreen(
                state =
                    WelcomeUiState(
                            tunnelState = tunnelState,
                            accountNumber = null,
                            deviceName = null,
                            showSitePayment = false,
                            verificationPending = false,
                        )
                        .toLc(),
                onDisconnectClick = clickHandler,
            )

            // Act
            onNodeWithText("Disconnect").performClick()

            // Assert
            verify { clickHandler() }
        }
}
