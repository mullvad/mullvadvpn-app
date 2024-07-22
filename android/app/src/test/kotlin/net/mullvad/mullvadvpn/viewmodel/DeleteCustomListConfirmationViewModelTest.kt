package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.dialog.DeleteCustomListNavArgs
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class DeleteCustomListConfirmationViewModelTest {
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()

    @Test
    fun `when successfully deleting a list should emit return with result side effect`() = runTest {
        // Arrange
        val deleted: Deleted = mockk()
        val customListName = CustomListName.fromString("name")
        val undo: CustomListAction.Create = mockk()
        val expectedResult =
            CustomListActionResultData.Deleted(customListName = customListName, undo = undo)
        every { deleted.name } returns customListName
        every { deleted.undo } returns undo
        val viewModel = createViewModel()
        coEvery { mockCustomListActionUseCase(any<CustomListAction.Delete>()) } returns
            deleted.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.deleteCustomList()
            val sideEffect = awaitItem()
            assertIs<DeleteCustomListConfirmationSideEffect.ReturnWithResult>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    private fun createViewModel() =
        DeleteCustomListConfirmationViewModel(
            customListActionUseCase = mockCustomListActionUseCase,
            savedStateHandle =
                DeleteCustomListNavArgs(
                        customListId = CustomListId("1"),
                        name = CustomListName.fromString("asdf")
                    )
                    .toSavedStateHandle()
        )
}
