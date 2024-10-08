package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.compose.dialog.EditCustomListNameNavArgs
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.RenameError
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class EditCustomListNameDialogViewModelTest {
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()

    @Test
    fun `when successfully renamed list should emit return with result side effect`() = runTest {
        // Arrange
        val renamed: Renamed = mockk()
        val customListId = CustomListId("id")
        val customListName = CustomListName.fromString("list")
        val undo: CustomListAction.Rename = mockk()
        val expectedResult =
            CustomListActionResultData.Success.Renamed(newName = customListName, undo = undo)
        every { renamed.name } returns customListName
        every { renamed.undo } returns undo
        val viewModel = createViewModel(customListId, customListName.value)
        coEvery { mockCustomListActionUseCase(any<CustomListAction.Rename>()) } returns
            renamed.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.updateCustomListName(customListName.value)
            val sideEffect = awaitItem()
            assertIs<EditCustomListNameDialogSideEffect.ReturnWithResult>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    @Test
    fun `when failing to rename a list should update ui state with error`() = runTest {
        // Arrange
        val customListId = CustomListId("id2")
        val customListName = CustomListName.fromString("list2")
        val expectedError = RenameError(NameAlreadyExists(customListName))
        val viewModel = createViewModel(customListId, customListName.value)
        coEvery { mockCustomListActionUseCase(any<CustomListAction.Rename>()) } returns
            expectedError.left()

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            viewModel.updateCustomListName(customListName.value)
            assertEquals(expectedError, awaitItem().error)
        }
    }

    @Test
    fun `given error state when calling clear error then should update to state without error`() =
        runTest {
            // Arrange
            val customListId = CustomListId("id")
            val customListName = CustomListName.fromString("list")
            val expectedError = RenameError(NameAlreadyExists(customListName))
            val viewModel = createViewModel(customListId, customListName.value)
            coEvery { mockCustomListActionUseCase(any<CustomListAction.Rename>()) } returns
                expectedError.left()

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                viewModel.updateCustomListName(customListName.value)
                assertEquals(expectedError, awaitItem().error) // Showing error
            }
        }

    private fun createViewModel(customListId: CustomListId, initialName: String) =
        EditCustomListNameDialogViewModel(
            customListActionUseCase = mockCustomListActionUseCase,
            savedStateHandle =
                EditCustomListNameNavArgs(
                        customListId = customListId,
                        initialName = CustomListName.fromString(initialName),
                    )
                    .toSavedStateHandle(),
        )
}
