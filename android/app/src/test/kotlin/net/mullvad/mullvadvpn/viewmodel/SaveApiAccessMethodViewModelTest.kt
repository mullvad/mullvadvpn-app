package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.dialog.SaveApiAccessMethodNavArgs
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.state.TestApiAccessMethodState
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.UnknownApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SaveApiAccessMethodViewModelTest {
    private val mockApiAccessRepository: ApiAccessRepository = mockk()

    private lateinit var saveApiAccessMethodViewModel: SaveApiAccessMethodViewModel

    @Test
    fun `when testing and updating an existing method successfully should do the correct steps`() =
        runTest {
            // Arrange
            val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val apiAccessMethodName = ApiAccessMethodName.fromString("Name")
            val customProxy = mockk<ApiAccessMethod.CustomProxy>()
            coEvery { mockApiAccessRepository.testCustomApiAccessMethod(customProxy) } returns
                Unit.right()
            coEvery {
                mockApiAccessRepository.updateApiAccessMethod(
                    apiAccessMethodId,
                    apiAccessMethodName,
                    customProxy,
                )
            } returns Unit.right()
            createSaveApiAccessMethodViewModel(
                apiAccessMethodId = apiAccessMethodId,
                apiAccessMethodName = apiAccessMethodName,
                customProxy = customProxy,
            )

            // Act, Assert
            saveApiAccessMethodViewModel.uiState.test {
                // After successful test
                assertEquals(
                    SaveApiAccessMethodUiState(
                        testingState = TestApiAccessMethodState.Result.Successful,
                        isSaving = true,
                    ),
                    awaitItem(),
                )
            }
            saveApiAccessMethodViewModel.uiSideEffect.test {
                // Check for successful creation
                assertEquals(
                    SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod,
                    awaitItem(),
                )
            }
        }

    @Test
    fun `when testing api access method fail should update ui state`() = runTest {
        // Arrange
        val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        val apiAccessMethodName = ApiAccessMethodName.fromString("Name")
        val customProxy = mockk<ApiAccessMethod.CustomProxy>()
        coEvery { mockApiAccessRepository.testCustomApiAccessMethod(customProxy) } returns
            TestApiAccessMethodError.CouldNotAccess.left()
        createSaveApiAccessMethodViewModel(
            apiAccessMethodId = apiAccessMethodId,
            apiAccessMethodName = apiAccessMethodName,
            customProxy = customProxy,
        )

        // Act, Assert
        saveApiAccessMethodViewModel.uiState.test {
            assertEquals(
                SaveApiAccessMethodUiState(
                    testingState = TestApiAccessMethodState.Result.Failure,
                    isSaving = false,
                ),
                awaitItem(),
            )
        }
    }

    @Test
    fun `when saving existing api access method after failure should update ui state`() = runTest {
        // Arrange
        val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        val apiAccessMethodName = ApiAccessMethodName.fromString("Name")
        val customProxy = mockk<ApiAccessMethod.CustomProxy>()
        coEvery { mockApiAccessRepository.testCustomApiAccessMethod(customProxy) } returns
            TestApiAccessMethodError.CouldNotAccess.left()
        coEvery {
            mockApiAccessRepository.updateApiAccessMethod(
                apiAccessMethodId,
                apiAccessMethodName,
                customProxy,
            )
        } returns Unit.right()
        createSaveApiAccessMethodViewModel(
            apiAccessMethodId = apiAccessMethodId,
            apiAccessMethodName = apiAccessMethodName,
            customProxy = customProxy,
        )

        // Act, Assert
        saveApiAccessMethodViewModel.uiState.test {
            // After successful test
            assertEquals(
                SaveApiAccessMethodUiState(
                    testingState = TestApiAccessMethodState.Result.Failure,
                    isSaving = false,
                ),
                awaitItem(),
            )
            saveApiAccessMethodViewModel.save()
            // Saving
            assertEquals(
                SaveApiAccessMethodUiState(
                    testingState = TestApiAccessMethodState.Result.Failure,
                    isSaving = true,
                ),
                awaitItem(),
            )
        }
        saveApiAccessMethodViewModel.uiSideEffect.test {
            // Check for successful creation
            assertEquals(SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod, awaitItem())
        }
    }

    @Test
    fun `when saving is not successful should return side effect failure`() = runTest {
        // Arrange
        val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        val apiAccessMethodName = ApiAccessMethodName.fromString("Name")
        val customProxy = mockk<ApiAccessMethod.CustomProxy>()
        coEvery { mockApiAccessRepository.testCustomApiAccessMethod(customProxy) } returns
            Unit.right()
        coEvery {
            mockApiAccessRepository.updateApiAccessMethod(
                apiAccessMethodId,
                apiAccessMethodName,
                customProxy,
            )
        } returns UnknownApiAccessMethodError(Throwable()).left()
        createSaveApiAccessMethodViewModel(
            apiAccessMethodId = apiAccessMethodId,
            apiAccessMethodName = apiAccessMethodName,
            customProxy = customProxy,
        )

        // Act, Assert
        saveApiAccessMethodViewModel.uiSideEffect.test {
            assertEquals(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod, awaitItem())
        }
    }

    @Test
    fun `when saving a new api access method should call addApiAccessMethod`() = runTest {
        // Arrange
        val apiAccessMethodId = null
        val apiAccessMethodName = ApiAccessMethodName.fromString("Name")
        val customProxy = mockk<ApiAccessMethod.CustomProxy>()
        coEvery { mockApiAccessRepository.testCustomApiAccessMethod(customProxy) } returns
            Unit.right()
        coEvery {
            mockApiAccessRepository.addApiAccessMethod(
                NewAccessMethodSetting(
                    name = apiAccessMethodName,
                    enabled = true,
                    apiAccessMethod = customProxy,
                )
            )
        } returns ApiAccessMethodId.fromString(UUID).right()
        createSaveApiAccessMethodViewModel(
            apiAccessMethodId = apiAccessMethodId,
            apiAccessMethodName = apiAccessMethodName,
            customProxy = customProxy,
        )

        // Assert
        coVerify(exactly = 1) {
            mockApiAccessRepository.addApiAccessMethod(
                NewAccessMethodSetting(
                    name = apiAccessMethodName,
                    enabled = true,
                    apiAccessMethod = customProxy,
                )
            )
        }
    }

    private fun createSaveApiAccessMethodViewModel(
        apiAccessMethodId: ApiAccessMethodId?,
        apiAccessMethodName: ApiAccessMethodName,
        customProxy: ApiAccessMethod.CustomProxy,
    ) {
        saveApiAccessMethodViewModel =
            SaveApiAccessMethodViewModel(
                apiAccessRepository = mockApiAccessRepository,
                savedStateHandle =
                    SaveApiAccessMethodNavArgs(
                            id = apiAccessMethodId,
                            name = apiAccessMethodName,
                            customProxy = customProxy,
                        )
                        .toSavedStateHandle(),
            )
    }
}
