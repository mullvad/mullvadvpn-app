package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DIRECT_ACCESS_METHOD
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_LIST_INFO_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ApiAccessListScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initScreen(
        state: ApiAccessListUiState = ApiAccessListUiState(),
        onAddMethodClick: () -> Unit = {},
        onApiAccessMethodClick: (apiAccessMethodSetting: ApiAccessMethodSetting) -> Unit = {},
        onApiAccessInfoClick: () -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            ApiAccessListScreen(
                state = state,
                onAddMethodClick = onAddMethodClick,
                onApiAccessMethodClick = onApiAccessMethodClick,
                onApiAccessInfoClick = onApiAccessInfoClick,
                onBackClick = onBackClick,
            )
        }
    }

    @Test
    fun shouldShowCurrentApiAccessName() =
        composeExtension.use {
            // Arrange
            val currentApiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state = ApiAccessListUiState(currentApiAccessMethodSetting = currentApiAccessMethod)
            )

            // Assert
            onNodeWithText("Current: ${currentApiAccessMethod.name}")
        }

    @Test
    fun shouldShowApiAccessNameAndStatusInList() =
        composeExtension.use {
            // Arrange
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            initScreen(
                state = ApiAccessListUiState(apiAccessMethodSettings = listOf(apiAccessMethod))
            )

            // Assert
            onNodeWithText(apiAccessMethod.name.value)
            onNodeWithText("On")
        }

    @Test
    fun whenClickingOnAddMethodShouldCallOnAddMethodClicked() =
        composeExtension.use {
            // Arrange
            val onAddMethodClick: () -> Unit = mockk(relaxed = true)
            initScreen(state = ApiAccessListUiState(), onAddMethodClick = onAddMethodClick)

            // Act
            onNodeWithText("Add").performClick()

            // Assert
            verify { onAddMethodClick() }
        }

    @Test
    fun whenClickingOnInfoButtonShouldCallOnApiAccessInfoClick() =
        composeExtension.use {
            // Arrange
            val onApiAccessInfoClick: () -> Unit = mockk(relaxed = true)
            initScreen(state = ApiAccessListUiState(), onApiAccessInfoClick = onApiAccessInfoClick)

            // Act
            onNodeWithTag(API_ACCESS_LIST_INFO_TEST_TAG).performClick()

            // Assert
            verify { onApiAccessInfoClick() }
        }

    @Test
    fun whenClickingOnApiAccessMethodShouldCallOnApiAccessMethodClickWithCorrectAccessMethod() =
        composeExtension.use {
            // Arrange
            val apiAccessMethod = DIRECT_ACCESS_METHOD
            val onApiAccessMethodClick: (ApiAccessMethodSetting) -> Unit = mockk(relaxed = true)
            initScreen(
                state = ApiAccessListUiState(apiAccessMethodSettings = listOf(apiAccessMethod)),
                onApiAccessMethodClick = onApiAccessMethodClick,
            )

            // Act
            onNodeWithText(apiAccessMethod.name.value).performClick()

            // Assert
            verify { onApiAccessMethodClick(apiAccessMethod) }
        }
}
