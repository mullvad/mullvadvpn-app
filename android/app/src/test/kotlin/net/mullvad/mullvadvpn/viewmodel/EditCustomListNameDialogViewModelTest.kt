package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class EditCustomListNameDialogViewModelTest {
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()

    @Test
    fun `when successfully renamed list should emit return with result side effect`() = runTest {
        // Arrange
        val expectedResult: CustomListResult.Renamed = mockk()
        val customListId = CustomListId("id")
        val customListName = "list"
        val viewModel = createViewModel(customListId, customListName)
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
        } returns expectedResult.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.updateCustomListName(customListName)
            val sideEffect = awaitItem()
            assertIs<EditCustomListNameDialogSideEffect.ReturnWithResult>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    @Test
    fun `when failing to rename a list should update ui state with error`() = runTest {
        // Arrange
        val customListId = CustomListId("id2")
        val customListName = "list2"
        val expectedError = UpdateCustomListError.NameAlreadyExists(customListName)
        val viewModel = createViewModel(customListId, customListName)
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
        } returns expectedError.left()

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            viewModel.updateCustomListName(customListName)
            assertEquals(expectedError, awaitItem().error)
        }
    }

    @Test
    fun `given error state when calling clear error then should update to state without error`() =
        runTest {
            // Arrange
            val customListId = CustomListId("id")
            val customListName = "list"
            val expectedError = UpdateCustomListError.NameAlreadyExists(customListName)
            val viewModel = createViewModel(customListId, customListName)
            coEvery {
                mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
            } returns expectedError.left()

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                viewModel.updateCustomListName(customListName)
                assertEquals(expectedError, awaitItem().error) // Showing error
                viewModel.clearError()
                assertNull(awaitItem().error)
            }
        }

    private fun createViewModel(customListId: CustomListId, initialName: String) =
        EditCustomListNameDialogViewModel(
            customListId = customListId,
            initialName = initialName,
            customListActionUseCase = mockCustomListActionUseCase
        )
}
