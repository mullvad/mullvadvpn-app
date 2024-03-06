package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsException
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
        val customListId = "id"
        val customListName = "list"
        val viewModel = createViewModel(customListId, customListName)
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
        } returns Result.success(expectedResult)

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.updateCustomListName(customListName)
            val sideEffect = awaitItem()
            assertIs<EditCustomListNameDialogSideEffect.ReturnWithResult>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    @Test
    fun `when failing to creating a list should update ui state with error`() = runTest {
        // Arrange
        val expectedError = CustomListsError.CustomListExists
        val customListId = "id2"
        val customListName = "list2"
        val viewModel = createViewModel(customListId, customListName)
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
        } returns Result.failure(CustomListsException(expectedError))

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
            val expectedError = CustomListsError.CustomListExists
            val customListId = "id"
            val customListName = "list"
            val viewModel = createViewModel(customListId, customListName)
            coEvery {
                mockCustomListActionUseCase.performAction(any<CustomListAction.Rename>())
            } returns Result.failure(CustomListsException(expectedError))

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                viewModel.updateCustomListName(customListName)
                assertEquals(expectedError, awaitItem().error) // Showing error
                viewModel.clearError()
                assertNull(awaitItem().error)
            }
        }

    private fun createViewModel(customListId: String, initialName: String) =
        EditCustomListNameDialogViewModel(
            customListId = customListId,
            initialName = initialName,
            customListActionUseCase = mockCustomListActionUseCase
        )
}
