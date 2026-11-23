package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_ADD_TIME_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_LATER_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_NEXT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_SKIP_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class OnboardingScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.setAddTimeScreen(
        state: AddTimeOnboardingUiState = AddTimeOnboardingUiState(),
        onAddTimeClick: () -> Unit = {},
        onPaymentInfoClick: (PaymentStatus) -> Unit = {},
    ) = setContentWithTheme {
        AddTimeOnboardingScreen(
            state = state,
            onPaymentInfoClick = onPaymentInfoClick,
            onBackClick = {},
            onAddTimeClick = onAddTimeClick,
        )
    }

    @Test
    fun testAddTimeButton() = composeExtension.use {
        // Arrange
        val onAddTimeClick: () -> Unit = mockk(relaxed = true)
        setAddTimeScreen(onAddTimeClick = onAddTimeClick)

        // Act
        onNodeWithTag(ONBOARDING_ADD_TIME_BUTTON_TEST_TAG).performClick()

        // Assert
        onNodeWithTag(ONBOARDING_ADD_TIME_BUTTON_TEST_TAG).assertExists()
        verify(exactly = 1) { onAddTimeClick() }
    }

    @Test
    fun testShowPendingPaymentInfoDialog() = composeExtension.use {
        // Arrange
        val onPaymentInfoClick: (PaymentStatus) -> Unit = mockk(relaxed = true)
        setAddTimeScreen(
            state = AddTimeOnboardingUiState(paymentStatus = PaymentStatus.PURCHASED_UNVERIFIED),
            onPaymentInfoClick = onPaymentInfoClick,
        )

        // Act
        onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

        // Assert
        onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).assertExists()
        verify(exactly = 1) { onPaymentInfoClick(PaymentStatus.PURCHASED_UNVERIFIED) }
    }

    @Test
    fun testShowVerificationInProgress() = composeExtension.use {
        // Arrange
        setAddTimeScreen(state = AddTimeOnboardingUiState(paymentStatus = PaymentStatus.PENDING))

        // Assert
        onNodeWithText("Google Play payment pending").assertExists()
    }

    @Test
    fun testShowVerificationFailed() = composeExtension.use {
        // Arrange
        setAddTimeScreen(
            state = AddTimeOnboardingUiState(paymentStatus = PaymentStatus.PURCHASED_UNVERIFIED),
        )

        // Assert
        onNodeWithText("Google Play payment verification failed").assertExists()
    }

    @Test
    fun testHidePaymentInfoWhenNoPendingPayment() = composeExtension.use {
        // Arrange
        setAddTimeScreen()

        // Assert
        onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).assertDoesNotExist()
    }

    @Test
    fun testGoodToKnowNextAndSkip() = composeExtension.use {
        // Arrange
        val onNextClick: () -> Unit = mockk(relaxed = true)
        val onSkipClick: () -> Unit = mockk(relaxed = true)
        setContentWithTheme {
            GoodToKnowOnboardingScreen(
                state = GoodToKnowUiState(GoodToKnowOnboardingPage.BLOCKED_CONTENT),
                onBackClick = null,
                onNextClick = onNextClick,
                onSkipClick = onSkipClick,
            )
        }

        // Act
        onNodeWithTag(ONBOARDING_NEXT_BUTTON_TEST_TAG).performClick()
        onNodeWithTag(ONBOARDING_SKIP_BUTTON_TEST_TAG).performClick()

        // Assert
        onNodeWithText("Good to know:", substring = true).assertExists()
        onNodeWithText("Blocked content", substring = true).assertExists()
        verify(exactly = 1) { onNextClick() }
        verify(exactly = 1) { onSkipClick() }
    }

    @Test
    fun testReadyToConnectButtons() = composeExtension.use {
        // Arrange
        val onConnectClick: () -> Unit = mockk(relaxed = true)
        val onLaterClick: () -> Unit = mockk(relaxed = true)
        setContentWithTheme {
            ReadyToConnectOnboardingScreen(
                state = ReadyToConnectUiState(selectedLocationTitle = "Sweden"),
                onBackClick = {},
                onConnectClick = onConnectClick,
                onLaterClick = onLaterClick,
            )
        }

        // Act
        onNodeWithTag(ONBOARDING_CONNECT_BUTTON_TEST_TAG).performClick()
        onNodeWithTag(ONBOARDING_LATER_BUTTON_TEST_TAG).performClick()

        // Assert
        onNodeWithText("Sweden").assertExists()
        verify(exactly = 1) { onConnectClick() }
        verify(exactly = 1) { onLaterClick() }
    }
}
