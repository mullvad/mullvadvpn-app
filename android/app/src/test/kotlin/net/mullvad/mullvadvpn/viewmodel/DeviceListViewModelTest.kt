package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.DeviceListNavArgs
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeleteDeviceError
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class DeviceListViewModelTest {

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockSavedStateHandle: SavedStateHandle = mockk(relaxed = true)

    private val testAccountNumber = AccountNumber("1234567890123456")
    private val testDeviceId1 = DeviceId.fromString("12345678-1234-5678-1234-567812345678")
    private val testDeviceId2 = DeviceId.fromString("87654321-1234-5678-1234-567812345678")

    private val testDevice1 =
        Device(
            id = testDeviceId1,
            name = "Device 1",
            creationDate = ZonedDateTime.now().minusSeconds(100),
        )

    private val testDevice2 =
        Device(
            id = testDeviceId2,
            name = "Device 2",
            creationDate = ZonedDateTime.now().minusSeconds(200),
        )
    private val testDeviceList = listOf(testDevice1, testDevice2)

    private lateinit var viewModel: DeviceListViewModel

    @BeforeEach
    fun setup() {
        // Mock SavedStateHandle to return the account number
        every { mockSavedStateHandle.get<AccountNumber>("accountNumber") } returns testAccountNumber

        // Mock successful device list fetch by default
        coEvery { mockDeviceRepository.deviceList(testAccountNumber) } returns
            testDeviceList.right()

        viewModel =
            DeviceListViewModel(
                deviceRepository = mockDeviceRepository,
                dispatcher = UnconfinedTestDispatcher(),
                savedStateHandle =
                    DeviceListNavArgs(accountNumber = testAccountNumber).toSavedStateHandle(),
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be Loading followed by Content`() = runTest {
        // Initial state is Loading
        assertIs<DeviceListUiState.Loading>(viewModel.uiState.value)

        viewModel.uiState.test {
            val contentState = awaitItem()
            assertIs<DeviceListUiState.Content>(contentState)
            assertEquals(2, contentState.devices.size)
        }
    }

    @Test
    fun `fetchDevices should update state to Error on failure`() = runTest {
        val error = GetDeviceListError.Unknown(RuntimeException("Network failed"))
        coEvery { mockDeviceRepository.deviceList(testAccountNumber) } returns error.left()

        viewModel.uiState.test {
            val errorState = awaitItem()
            assertIs<DeviceListUiState.Error>(errorState)
            assertEquals(error, errorState.error)
        }
    }

    @Test
    fun `removeDevice should update device login state and remove device on success`() = runTest {
        val deviceToRemove = testDeviceId1
        coEvery { mockDeviceRepository.removeDevice(testAccountNumber, deviceToRemove) } coAnswers
            {
                // The delay here will cause the call to deviceRepository.removeDevice() to suspend.
                // At the time removeDevice() suspends, device.isLoading should be true.
                // When we suspend, control is yielded from the launched coroutine back to the test,
                // and we are able to call awaitItem() and test that the device state is loading.
                delay(1)
                Unit.right()
            }

        viewModel.uiState.test {
            assertIs<DeviceListUiState.Content>(awaitItem())

            // Act: remove device
            viewModel.removeDevice(deviceToRemove)

            // State reflects loading state for the specific device
            val state = awaitItem()
            assertIs<DeviceListUiState.Content>(state)

            // Ensure that the removing device state is loading as it has not ye been removed
            assertTrue(state.devices.first { it.device.id == deviceToRemove }.isLoading)

            // State reflects removal
            val finalState = awaitItem()
            assertIs<DeviceListUiState.Content>(finalState)
            assertEquals(1, finalState.devices.size)
            assertEquals(testDeviceId2, finalState.devices[0].device.id)
        }
    }

    @Test
    fun `removeDevice should emit FailedToRemoveDevice side effect and refresh list on failure`() =
        runTest {
            val deviceToRemove = testDeviceId1
            val removeError = DeleteDeviceError.Unknown(RuntimeException("Failed to remove"))
            // Mock remove failure
            coEvery {
                mockDeviceRepository.removeDevice(testAccountNumber, deviceToRemove)
            } coAnswers
                {
                    delay(1)
                    removeError.left()
                }
            // Mock subsequent list refresh success
            coEvery { mockDeviceRepository.deviceList(testAccountNumber) } returns
                testDeviceList.right()

            // Initial state check
            viewModel.uiState.test {
                assertIs<DeviceListUiState.Content>(awaitItem())

                // Act: Remove device (which will fail)
                viewModel.removeDevice(deviceToRemove)

                // State reflects loading state for the specific device
                val state = awaitItem()
                assertIs<DeviceListUiState.Content>(state)
                assertTrue(state.devices.first { it.device.id == deviceToRemove }.isLoading)

                viewModel.uiSideEffect.test {
                    // Check side effect
                    assertEquals(DeviceListSideEffect.FailedToRemoveDevice, awaitItem())
                }

                // State reflects end of loading (device still present) and list refreshed
                val finalState = awaitItem()
                assertIs<DeviceListUiState.Content>(finalState)
                assertEquals(2, finalState.devices.size) // Both devices still there
                // Loading state should be false again
                assertTrue(finalState.devices.none { it.isLoading })
            }
        }

    @Test
    fun `continueToLogin should emit NavigateToLogin side effect`() = runTest {
        viewModel.uiSideEffect.test {
            viewModel.continueToLogin()
            assertEquals(
                DeviceListSideEffect.NavigateToLogin(accountNumber = testAccountNumber),
                awaitItem(),
            )
        }
    }
}
