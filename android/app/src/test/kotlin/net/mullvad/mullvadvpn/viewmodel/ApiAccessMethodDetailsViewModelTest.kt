package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.Either
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import java.time.Duration
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.time.delay
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.UnknownApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.util.delayAtLeast
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ApiAccessMethodDetailsViewModelTest {
    private val mockApiAccessRepository: ApiAccessRepository = mockk()
    private val apiAccessMethodId = ApiAccessMethodId.fromString(UUID)

    private lateinit var apiAccessMethodDetailsViewModel: ApiAccessMethodDetailsViewModel

    private val accessMethodFlow = MutableStateFlow<ApiAccessMethodSetting>(mockk(relaxed = true))
    private val enabledMethodsFlow = MutableStateFlow<List<ApiAccessMethodSetting>>(emptyList())
    private val currentAccessMethodFlow = MutableStateFlow<ApiAccessMethodSetting?>(null)

    @BeforeEach
    fun setUp() {
        mockkStatic(DELAY_UTIL)
        every { mockApiAccessRepository.apiAccessMethodSettingById(apiAccessMethodId) } returns
            accessMethodFlow
        every { mockApiAccessRepository.enabledApiAccessMethods() } returns enabledMethodsFlow
        every { mockApiAccessRepository.currentAccessMethod } returns currentAccessMethodFlow

        apiAccessMethodDetailsViewModel =
            ApiAccessMethodDetailsViewModel(
                apiAccessMethodId = apiAccessMethodId,
                apiAccessRepository = mockApiAccessRepository
            )
    }

    @Test
    fun `when calling set current method and testing is successful should call set method`() =
        runTest {
            // Arrange
            coEvery { mockApiAccessRepository.testApiAccessMethodById(apiAccessMethodId) } returns
                Unit.right()
            coEvery { mockApiAccessRepository.setCurrentApiAccessMethod(any()) } returns
                Unit.right()
            coEvery { delayAtLeast<Either<TestApiAccessMethodError, Unit>>(any(), any()) } returns
                Unit.right()

            // Act
            apiAccessMethodDetailsViewModel.setCurrentMethod()

            // Assert
            coVerify(exactly = 1) {
                mockApiAccessRepository.setCurrentApiAccessMethod(apiAccessMethodId)
            }
        }

    @Test
    fun `when calling set current method and testing is not successful should not call set method`() =
        runTest {
            // Arrange
            coEvery { mockApiAccessRepository.testApiAccessMethodById(apiAccessMethodId) } returns
                TestApiAccessMethodError.CouldNotAccess.left()
            coEvery { mockApiAccessRepository.setCurrentApiAccessMethod(any()) } returns
                Unit.right()

            // Act
            apiAccessMethodDetailsViewModel.setCurrentMethod()

            // Assert
            coVerify(exactly = 0) {
                mockApiAccessRepository.setCurrentApiAccessMethod(apiAccessMethodId)
            }
        }

    @Test
    fun `when testing method should update is testing access method to true`() = runTest {
        // Arrange
        coEvery { mockApiAccessRepository.testApiAccessMethodById(apiAccessMethodId) } coAnswers
            {
                // Added so that the state gets updated
                delay(Duration.ofMillis(1))
                Unit.right()
            }

        // Act, Assert
        apiAccessMethodDetailsViewModel.uiState.test {
            // Default item
            awaitItem()
            apiAccessMethodDetailsViewModel.testMethod()
            val result = awaitItem()
            assertIs<ApiAccessMethodDetailsUiState.Content>(result)
            assertEquals(true, result.isTestingAccessMethod)
        }
    }

    @Test
    fun `when testing method is successful should send side effect api reached`() = runTest {
        // Arrange
        coEvery { mockApiAccessRepository.testApiAccessMethodById(apiAccessMethodId) } returns
            Unit.right()

        // Act, Assert
        apiAccessMethodDetailsViewModel.uiSideEffect.test {
            apiAccessMethodDetailsViewModel.testMethod()
            val result = awaitItem()
            assertIs<ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult>(result)
            assertEquals(true, result.successful)
        }
    }

    @Test
    fun `when testing method is not successful should send side effect api not reached`() =
        runTest {
            // Arrange
            coEvery { mockApiAccessRepository.testApiAccessMethodById(apiAccessMethodId) } returns
                TestApiAccessMethodError.CouldNotAccess.left()

            // Act, Assert
            apiAccessMethodDetailsViewModel.uiSideEffect.test {
                apiAccessMethodDetailsViewModel.testMethod()
                val result = awaitItem()
                assertIs<ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult>(result)
                assertEquals(false, result.successful)
            }
        }

    @Test
    fun `when enable access method is successful nothing should happen`() = runTest {
        // Arrange
        coEvery {
            mockApiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, true)
        } returns Unit.right()

        // Act, Assert
        apiAccessMethodDetailsViewModel.uiSideEffect.test {
            apiAccessMethodDetailsViewModel.setEnableMethod(true)
            expectNoEvents()
        }
    }

    @Test
    fun `when enable access method is not successful should show error`() = runTest {
        // Arrange
        coEvery {
            mockApiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, true)
        } returns UnknownApiAccessMethodError(Throwable()).left()

        // Act, Assert
        apiAccessMethodDetailsViewModel.uiSideEffect.test {
            apiAccessMethodDetailsViewModel.setEnableMethod(true)
            assertEquals(ApiAccessMethodDetailsSideEffect.GenericError, awaitItem())
        }
    }

    @Test
    fun `calling open edit page should return side effect with id`() = runTest {
        // Act, Assert
        apiAccessMethodDetailsViewModel.uiSideEffect.test {
            apiAccessMethodDetailsViewModel.openEditPage()
            assertEquals(
                ApiAccessMethodDetailsSideEffect.OpenEditPage(apiAccessMethodId),
                awaitItem()
            )
        }
    }

    companion object {
        private const val DELAY_UTIL = "net.mullvad.mullvadvpn.util.DelayKt"
    }
}
