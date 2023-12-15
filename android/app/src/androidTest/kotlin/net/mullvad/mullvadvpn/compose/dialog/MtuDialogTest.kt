package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import de.mannodermaus.junit5.compose.createComposeExtension
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class MtuDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @SuppressLint("ComposableNaming")
    @Composable
    private fun testMtuDialog(
        mtuInitial: Int? = null,
        onSaveMtu: (Int) -> Unit = { _ -> },
        onResetMtu: () -> Unit = {},
        onDismiss: () -> Unit = {},
    ) {
        MtuDialog(
            mtuInitial = mtuInitial,
            onSaveMtu = onSaveMtu,
            onResetMtu = onResetMtu,
            onDismiss = onDismiss
        )
    }

    @Test
    fun testMtuDialogWithDefaultValue() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { testMtuDialog() }

            // Assert
            onNodeWithText(EMPTY_STRING).assertExists()
        }

    @Test
    fun testMtuDialogWithEditValue() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testMtuDialog(
                    mtuInitial = VALID_DUMMY_MTU_VALUE,
                )
            }

            // Assert
            onNodeWithText(VALID_DUMMY_MTU_VALUE.toString()).assertExists()
        }

    @Test
    fun testMtuDialogTextInput() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testMtuDialog(
                    null,
                )
            }

            // Act
            onNodeWithText(EMPTY_STRING).performTextInput(VALID_DUMMY_MTU_VALUE.toString())

            // Assert
            onNodeWithText(VALID_DUMMY_MTU_VALUE.toString()).assertExists()
        }

    @Test
    fun testMtuDialogSubmitOfValidValue() =
        composeExtension.use {
            // Arrange
            val mockedSubmitHandler: (Int) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                testMtuDialog(
                    VALID_DUMMY_MTU_VALUE,
                    onSaveMtu = mockedSubmitHandler,
                )
            }

            // Act
            onNodeWithText("Submit").assertIsEnabled().performClick()

            // Assert
            verify { mockedSubmitHandler.invoke(VALID_DUMMY_MTU_VALUE) }
        }

    @Test
    fun testMtuDialogSubmitButtonDisabledWhenInvalidInput() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testMtuDialog(
                    INVALID_DUMMY_MTU_VALUE,
                )
            }

            // Assert
            onNodeWithText("Submit").assertIsNotEnabled()
        }

    @Test
    fun testMtuDialogResetClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                testMtuDialog(
                    onResetMtu = mockedClickHandler,
                )
            }

            // Act
            onNodeWithText("Reset to default").performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }

    @Test
    fun testMtuDialogCancelClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                testMtuDialog(
                    onDismiss = mockedClickHandler,
                )
            }

            // Assert
            onNodeWithText("Cancel").performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }

    companion object {
        private const val EMPTY_STRING = ""
        private const val VALID_DUMMY_MTU_VALUE = 1337
        private const val INVALID_DUMMY_MTU_VALUE = 1111
    }
}
