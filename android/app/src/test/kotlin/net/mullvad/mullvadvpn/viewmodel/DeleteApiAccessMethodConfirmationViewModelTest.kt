package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.dialog.DeleteApiAccessMethodNavArgs
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.RemoveApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class DeleteApiAccessMethodConfirmationViewModelTest {

    private val mockApiAccessRepository: ApiAccessRepository = mockk()
    private lateinit var deleteApiAccessMethodConfirmationViewModel:
        DeleteApiAccessMethodConfirmationViewModel

    @BeforeEach
    fun setUp() {
        val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)

        deleteApiAccessMethodConfirmationViewModel =
            DeleteApiAccessMethodConfirmationViewModel(
                apiAccessRepository = mockApiAccessRepository,
                savedStateHandle =
                    DeleteApiAccessMethodNavArgs(apiAccessMethodId = apiAccessMethodId)
                        .toSavedStateHandle(),
            )
    }

    @Test
    fun `when deleting api access method is successful should update uiSideEffect`() = runTest {
        // Arrange
        coEvery { mockApiAccessRepository.removeApiAccessMethod(any()) } returns Unit.right()

        // Act, Assert
        deleteApiAccessMethodConfirmationViewModel.uiSideEffect.test {
            deleteApiAccessMethodConfirmationViewModel.deleteApiAccessMethod()
            val result = awaitItem()
            assertEquals(DeleteApiAccessMethodConfirmationSideEffect.Deleted, result)
        }
    }

    @Test
    fun `when deleting api access method is not successful should update ui state`() = runTest {
        // Arrange
        val error = RemoveApiAccessMethodError.Unknown(Throwable())
        coEvery { mockApiAccessRepository.removeApiAccessMethod(any()) } returns error.left()

        // Act, Assert
        deleteApiAccessMethodConfirmationViewModel.uiState.test {
            // Default item
            awaitItem()
            deleteApiAccessMethodConfirmationViewModel.deleteApiAccessMethod()
            val result = awaitItem().deleteError
            assertEquals(error, result)
        }
    }
}
