package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.CUSTOM_ACCESS_METHOD
import net.mullvad.mullvadvpn.compose.data.DIRECT_ACCESS_METHOD
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_DETAILS_EDIT_BUTTON
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_TEST_METHOD_BUTTON
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_USE_METHOD_BUTTON
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ApiAccessMethodDetailsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initScreen(
        state: ApiAccessMethodDetailsUiState,
        onEditMethodClicked: () -> Unit = {},
        onEnableClicked: (Boolean) -> Unit = {},
        onTestMethodClicked: () -> Unit = {},
        onUseMethodClicked: () -> Unit = {},
        onDeleteApiAccessMethodClicked: (ApiAccessMethodId) -> Unit = {},
        onNavigateToEncryptedDnsInfoDialog: () -> Unit = {},
        onBackClicked: () -> Unit = {},
    ) {
        setContentWithTheme {
            ApiAccessMethodDetailsScreen(
                state = state,
                onEditMethodClicked = onEditMethodClicked,
                onEnableClicked = onEnableClicked,
                onTestMethodClicked = onTestMethodClicked,
                onUseMethodClicked = onUseMethodClicked,
                onDeleteApiAccessMethodClicked = onDeleteApiAccessMethodClicked,
                onNavigateToEncryptedDnsInfoDialog = onNavigateToEncryptedDnsInfoDialog,
                onBackClicked = onBackClicked,
            )
        }
    }

    @Test
    fun whenApiAccessMethodIsNotEditableShouldNotShowDeleteAndEdit() =
        composeExtension.use {
            // Arrange
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = true,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithTag(API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG).assertDoesNotExist()
            onNodeWithTag(API_ACCESS_DETAILS_EDIT_BUTTON).assertDoesNotExist()
        }

    @Test
    fun whenApiAccessMethodIsNotDisableableShouldNotBeAbleDisable() =
        composeExtension.use {
            // Arrange
            val onEnableClicked: (Boolean) -> Unit = mockk(relaxed = true)
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = false,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    ),
                onEnableClicked = onEnableClicked,
            )

            // Act
            onNodeWithText("Enable method").performClick()

            // Assert
            onNodeWithText("At least one method needs to be enabled")
            verify(exactly = 0) { onEnableClicked(any()) }
        }

    @Test
    fun whenClickingOnDeleteMethodShouldCallOnDeleteApiAccessMethodClicked() =
        composeExtension.use {
            // Arrange
            val onDeleteApiAccessMethodClicked: (ApiAccessMethodId) -> Unit = mockk(relaxed = true)
            val apiAccessMethod = CUSTOM_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = false,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    ),
                onDeleteApiAccessMethodClicked = onDeleteApiAccessMethodClicked,
            )

            // Act
            onNodeWithTag(API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG).performClick()
            onNodeWithText("Delete method").performClick()

            // Assert
            verify(exactly = 1) { onDeleteApiAccessMethodClicked(apiAccessMethod.id) }
        }

    @Test
    fun whenClickingOnEditMethodShouldCallOnEditMethodClicked() =
        composeExtension.use {
            // Arrange
            val onEditMethodClicked: () -> Unit = mockk(relaxed = true)
            val apiAccessMethod = CUSTOM_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = false,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    ),
                onEditMethodClicked = onEditMethodClicked,
            )

            // Act
            onNodeWithTag(API_ACCESS_DETAILS_EDIT_BUTTON).performClick()

            // Assert
            verify(exactly = 1) { onEditMethodClicked() }
        }

    @Test
    fun whenClickingOnEnableMethodShouldCallOnEnableClicked() =
        composeExtension.use {
            // Arrange
            val onEnableClicked: (Boolean) -> Unit = mockk(relaxed = true)
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = true,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    ),
                onEnableClicked = onEnableClicked,
            )

            // Act
            onNodeWithText("Enable method").performClick()

            // Assert
            verify(exactly = 1) { onEnableClicked(false) }
        }

    @Test
    fun whenClickingOnTestMethodShouldCallOnTestMethodClicked() =
        composeExtension.use {
            // Arrange
            val onTestMethodClicked: () -> Unit = mockk(relaxed = true)
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = true,
                        isCurrentMethod = true,
                        isTestingAccessMethod = false,
                    ),
                onTestMethodClicked = onTestMethodClicked,
            )

            // Act
            onNodeWithTag(API_ACCESS_TEST_METHOD_BUTTON).performClick()

            // Assert
            verify(exactly = 1) { onTestMethodClicked() }
        }

    @Test
    fun whenClickingOnUseMethodShouldCallOnUseMethodClicked() =
        composeExtension.use {
            // Arrange
            val onUseMethodClicked: () -> Unit = mockk(relaxed = true)
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state =
                    ApiAccessMethodDetailsUiState.Content(
                        apiAccessMethodSetting = apiAccessMethod,
                        isDisableable = true,
                        isCurrentMethod = false,
                        isTestingAccessMethod = false,
                    ),
                onUseMethodClicked = onUseMethodClicked,
            )

            // Act
            onNodeWithTag(API_ACCESS_USE_METHOD_BUTTON).performClick()

            // Assert
            verify(exactly = 1) { onUseMethodClicked() }
        }
}
