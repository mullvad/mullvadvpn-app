package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.test.EDIT_API_ACCESS_NAME_INPUT
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class EditApiAccessMethodScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initScreen(
        state: EditApiAccessMethodUiState,
        onNameChanged: (String) -> Unit = {},
        onTypeSelected: (ApiAccessMethodTypes) -> Unit = {},
        onIpChanged: (String) -> Unit = {},
        onPortChanged: (String) -> Unit = {},
        onPasswordChanged: (String) -> Unit = {},
        onCipherChange: (Cipher) -> Unit = {},
        onToggleAuthenticationEnabled: (Boolean) -> Unit = {},
        onUsernameChanged: (String) -> Unit = {},
        onTestMethod: () -> Unit = {},
        onAddMethod: () -> Unit = {},
        onNavigateBack: () -> Unit = {},
    ) {
        setContentWithTheme {
            EditApiAccessMethodScreen(
                state = state,
                onNameChanged = onNameChanged,
                onTypeSelected = onTypeSelected,
                onIpChanged = onIpChanged,
                onPortChanged = onPortChanged,
                onPasswordChanged = onPasswordChanged,
                onCipherChange = onCipherChange,
                onToggleAuthenticationEnabled = onToggleAuthenticationEnabled,
                onUsernameChanged = onUsernameChanged,
                onTestMethod = onTestMethod,
                onAddMethod = onAddMethod,
                onNavigateBack = onNavigateBack,
            )
        }
    }

    @Test
    fun whenInEditModeAddButtonShouldSaySave() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = true,
                        formData = EditApiAccessFormData.empty(),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("Save").assertExists()
        }

    @Test
    fun whenNotInEditModeAddButtonShouldSayAdd() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData = EditApiAccessFormData.empty(),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("Add").assertExists()
        }

    @Test
    fun whenNameInputHasErrorShouldShowError() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData =
                            EditApiAccessFormData(
                                name = "",
                                nameError = InvalidDataError.NameError.Required,
                                serverIp = "",
                                username = "",
                                password = "",
                                port = "",
                            ),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("This field is required").assertExists()
        }

    @Test
    fun whenServerInputIsNotIpAddressShouldShowError() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData =
                            EditApiAccessFormData(
                                name = "",
                                serverIp = "123",
                                serverIpError = InvalidDataError.ServerIpError.Invalid,
                                username = "",
                                password = "",
                                port = "",
                            ),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("Please enter a valid IPv4 or IPv6 address").assertExists()
        }

    @Test
    fun whenPortInputIsNotWithinRangeShouldShowError() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData =
                            EditApiAccessFormData(
                                name = "",
                                serverIp = "",
                                username = "",
                                password = "",
                                port = "1111111111",
                                portError =
                                    InvalidDataError.PortError.Invalid(
                                        ParsePortError.OutOfRange(1111111111)
                                    ),
                            ),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("Please enter a valid remote server port").assertExists()
        }

    @Test
    fun whenNameInputChangesShouldCallOnNameChanged() =
        composeExtension.use {
            // Arrange
            val onNameChanged: (String) -> Unit = mockk(relaxed = true)
            val mockInput = "Name"
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData = EditApiAccessFormData.empty(),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    ),
                onNameChanged = onNameChanged,
            )

            // Act
            onNodeWithTag(EDIT_API_ACCESS_NAME_INPUT).performTextInput(mockInput)

            // Assert
            verify(exactly = 1) { onNameChanged(mockInput) }
        }

    @Test
    fun whenSocks5IsSelectedAndAuthenticationIsEnabledShouldShowUsernameAndPassword() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData =
                            EditApiAccessFormData(
                                name = "",
                                serverIp = "",
                                username = "",
                                password = "",
                                port = "",
                                enableAuthentication = true,
                                apiAccessMethodTypes = ApiAccessMethodTypes.SOCKS5_REMOTE,
                            ),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    )
            )

            // Assert
            onNodeWithText("Username").assertExists()
            onNodeWithText("Password").assertExists()
        }

    @Test
    fun whenClickingOnTestMethodButtonShouldCallOnTestMethod() =
        composeExtension.use {
            // Arrange
            val onTestMethod: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData = EditApiAccessFormData.empty(),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    ),
                onTestMethod = onTestMethod,
            )

            // Act
            onNodeWithText("Test method").performClick()

            // Assert
            verify(exactly = 1) { onTestMethod() }
        }

    @Test
    fun whenClickingOnAddMethodButtonShouldCallOnAddMethod() =
        composeExtension.use {
            // Arrange
            val onAddMethod: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    EditApiAccessMethodUiState.Content(
                        editMode = false,
                        formData = EditApiAccessFormData.empty(),
                        hasChanges = false,
                        isTestingApiAccessMethod = false,
                    ),
                onAddMethod = onAddMethod,
            )

            // Act
            onNodeWithText("Add").performClick()

            // Assert
            verify(exactly = 1) { onAddMethod() }
        }
}
