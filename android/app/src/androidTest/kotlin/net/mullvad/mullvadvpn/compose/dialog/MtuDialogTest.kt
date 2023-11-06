package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class MtuDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
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
    fun testMtuDialogWithDefaultValue() {
        // Arrange
        composeTestRule.setContentWithTheme { testMtuDialog() }

        // Assert
        composeTestRule.onNodeWithText(EMPTY_STRING).assertExists()
    }

    @Test
    fun testMtuDialogWithEditValue() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                mtuInitial = VALID_DUMMY_MTU_VALUE,
            )
        }

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE.toString()).assertExists()
    }

    @Test
    fun testMtuDialogTextInput() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                null,
            )
        }

        // Act
        composeTestRule
            .onNodeWithText(EMPTY_STRING)
            .performTextInput(VALID_DUMMY_MTU_VALUE.toString())

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE.toString()).assertExists()
    }

    @Test
    fun testMtuDialogSubmitOfValidValue() {
        // Arrange
        val mockedSubmitHandler: (Int) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                VALID_DUMMY_MTU_VALUE,
                onSaveMtu = mockedSubmitHandler,
            )
        }

        // Act
        composeTestRule.onNodeWithText("Submit").assertIsEnabled().performClick()

        // Assert
        verify { mockedSubmitHandler.invoke(VALID_DUMMY_MTU_VALUE) }
    }

    @Test
    fun testMtuDialogSubmitButtonDisabledWhenInvalidInput() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                INVALID_DUMMY_MTU_VALUE,
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    fun testMtuDialogResetClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                onResetMtu = mockedClickHandler,
            )
        }

        // Act
        composeTestRule.onNodeWithText("Reset to default").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testMtuDialogCancelClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            testMtuDialog(
                onDismiss = mockedClickHandler,
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Cancel").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    companion object {
        private const val EMPTY_STRING = ""
        private const val VALID_DUMMY_MTU_VALUE = 1337
        private const val INVALID_DUMMY_MTU_VALUE = 1111
    }
}
