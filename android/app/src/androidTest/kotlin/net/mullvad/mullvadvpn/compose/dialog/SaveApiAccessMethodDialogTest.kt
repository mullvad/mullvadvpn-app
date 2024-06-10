package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class SaveApiAccessMethodDialogTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @Test
    fun whenTestingInProgressShouldShowSpinnerWithCancelButton() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SaveApiAccessMethodDialog(
                    state =
                        SaveApiAccessMethodUiState(
                            testingState = TestApiAccessMethodState.Testing,
                            isSaving = false
                        )
                )
            }

            // Assert
            onNodeWithTag(SAVE_API_ACCESS_METHOD_LOADING_SPINNER_TEST_TAG).assertExists()
            onNodeWithTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG).assertExists()
        }

    @Test
    fun whenTestingFailedShouldShowSaveAndCancelButton() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SaveApiAccessMethodDialog(
                    state =
                        SaveApiAccessMethodUiState(
                            testingState = TestApiAccessMethodState.Result.Failure,
                            isSaving = false
                        )
                )
            }

            // Assert
            onNodeWithTag(SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG).assertExists()
            onNodeWithTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG).assertExists()
        }

    @Test
    fun whenTestingSuccessfulAndSavingShouldShowDisabledCancelButton() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SaveApiAccessMethodDialog(
                    state =
                        SaveApiAccessMethodUiState(
                            testingState = TestApiAccessMethodState.Result.Successful,
                            isSaving = true
                        )
                )
            }

            // Assert
            onNodeWithTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG).assertExists()
            onNodeWithTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG).assertIsNotEnabled()
        }

    @Test
    fun whenTestingInProgressAndClickingCancelShouldCallOnCancel() =
        composeExtension.use {
            // Arrange
            val onCancelClick: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SaveApiAccessMethodDialog(
                    state =
                        SaveApiAccessMethodUiState(
                            testingState = TestApiAccessMethodState.Testing,
                            isSaving = false
                        ),
                    onCancel = onCancelClick
                )
            }

            // Act
            onNodeWithTag(SAVE_API_ACCESS_METHOD_CANCEL_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { onCancelClick() }
        }

    @Test
    fun whenTestingFailedAndClickingSaveShouldCallOnSave() =
        composeExtension.use {
            // Arrange
            val onSaveClick: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SaveApiAccessMethodDialog(
                    state =
                        SaveApiAccessMethodUiState(
                            testingState = TestApiAccessMethodState.Result.Failure,
                            isSaving = false
                        ),
                    onSave = onSaveClick
                )
            }

            // Act
            onNodeWithTag(SAVE_API_ACCESS_METHOD_SAVE_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { onSaveClick() }
        }
}
