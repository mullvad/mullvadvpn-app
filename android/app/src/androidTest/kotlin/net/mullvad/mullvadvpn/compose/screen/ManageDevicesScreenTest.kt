package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.SnackbarHostState
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ManageDevicesItemUiState
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.util.withRole
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.util.Lce
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@ExperimentalTestApi
class ManageDevicesScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: Lce<ManageDevicesUiState, GetDeviceListError>,
        snackbarHostState: SnackbarHostState = SnackbarHostState(),
        onBackClick: () -> Unit = {},
        onTryAgainClicked: () -> Unit = {},
        navigateToRemoveDeviceConfirmationDialog: (device: Device) -> Unit = {},
    ) {
        setContentWithTheme {
            ManageDevicesScreen(
                state = state,
                snackbarHostState = snackbarHostState,
                onBackClick = onBackClick,
                onTryAgainClicked = onTryAgainClicked,
                navigateToRemoveDeviceConfirmationDialog = navigateToRemoveDeviceConfirmationDialog,
            )
        }
    }

    @Test
    fun loadingStateShowsProgressIndicator() {
        composeExtension.use {
            // Arrange
            initScreen(state = Lce.Loading)

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertIsDisplayed()
        }
    }

    @Test
    fun errorStateShowsErrorMessageAndTryAgainButton() {
        composeExtension.use {
            // Arrange
            val onTryAgainClicked: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = Lce.Error(GetDeviceListError.Unknown(Throwable("error"))),
                onTryAgainClicked = onTryAgainClicked,
            )

            // Assert
            onNodeWithText("Failed to fetch list of devices").assertIsDisplayed()
            onNodeWithText("Try again").assertIsDisplayed()
            onNodeWithText("Manage devices").assertIsDisplayed()

            // Act
            onNodeWithText("Try again").performClick()

            // Assert
            verify(exactly = 1) { onTryAgainClicked.invoke() }
        }
    }

    @Test
    fun contentStateShowsDeviceListCorrectly() {
        composeExtension.use {
            // Arrange
            val device1 =
                Device(
                    id = DeviceId.fromString("12345678-1234-5678-1234-567812345678"),
                    name = "Laptop",
                    creationDate = ZonedDateTime.now().minusSeconds(100),
                )
            val device2 =
                Device(
                    id = DeviceId.fromString("87654321-1234-5678-1234-567812345678"),
                    name = "My Phone",
                    creationDate = ZonedDateTime.now().minusSeconds(200),
                )

            val device3 =
                Device(
                    id = DeviceId.fromString("87654321-4321-5678-1234-567812345678"),
                    name = "Tablet",
                    creationDate = ZonedDateTime.now().minusSeconds(300),
                )

            val state =
                ManageDevicesUiState(
                    devices =
                        listOf(
                            ManageDevicesItemUiState(
                                device2,
                                isLoading = false,
                                isCurrentDevice = true,
                            ),
                            ManageDevicesItemUiState(
                                device1,
                                isLoading = false,
                                isCurrentDevice = false,
                            ),
                            ManageDevicesItemUiState(
                                device3,
                                isLoading = true,
                                isCurrentDevice = false,
                            ),
                        )
                )
            initScreen(state = Lce.Content(state))

            // Assert
            onNodeWithText("Manage devices").assertIsDisplayed()

            onNodeWithText("Laptop").assertIsDisplayed()
            onNodeWithText("Current device").assertIsDisplayed()
            onNodeWithText("My Phone").assertIsDisplayed()
            onNodeWithText("Tablet").assertIsDisplayed()

            // We should have 2 visible buttons (the navbar back button and the remove button for
            // device "Laptop"
            val buttons = onAllNodes(withRole(Role.Button))
            buttons.assertCountEquals(2)
            buttons[0].assertIsDisplayed()
            buttons[1].assertIsDisplayed()

            // Make sure the device that is loading is displaying the spinner
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertIsDisplayed()
        }
    }
}
