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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.DeviceListNavArgs
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.util.Lce
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class ManageDevicesViewModelTest {

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockSavedStateHandle: SavedStateHandle = mockk(relaxed = true)

    private val testAccountNumber = AccountNumber("1234567890123456")
    private val testDeviceId1 = DeviceId.fromString("12345678-1234-5678-1234-567812345678")
    private val testDeviceId2 = DeviceId.fromString("87654321-1234-5678-1234-567812345678")
    private val testDeviceId3 = DeviceId.fromString("87654321-4321-5678-1234-567812345678")

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

    private val testDevice3 =
        Device(
            id = testDeviceId3,
            name = "Device 3",
            creationDate = ZonedDateTime.now().minusSeconds(300),
        )
    private val testDeviceList = listOf(testDevice1, testDevice2, testDevice3)

    private val deviceState =
        DeviceState.LoggedIn(accountNumber = testAccountNumber, device = testDevice2)

    private lateinit var viewModel: ManageDevicesViewModel

    @BeforeEach
    fun setup() {
        // Mock SavedStateHandle to return the account number
        every { mockSavedStateHandle.get<AccountNumber>("accountNumber") } returns testAccountNumber

        // Mock successful device list fetch by default
        coEvery { mockDeviceRepository.deviceList(testAccountNumber) } returns
            testDeviceList.right()

        every { mockDeviceRepository.deviceState } returns MutableStateFlow(deviceState)

        viewModel =
            ManageDevicesViewModel(
                deviceRepository = mockDeviceRepository,
                deviceListViewModel =
                    DeviceListViewModel(
                        deviceRepository = mockDeviceRepository,
                        dispatcher = UnconfinedTestDispatcher(),
                        savedStateHandle =
                            DeviceListNavArgs(accountNumber = testAccountNumber)
                                .toSavedStateHandle(),
                    ),
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
        assertIs<Lce.Loading>(viewModel.uiState.value)

        viewModel.uiState.test {
            val contentState = awaitItem()
            assertIs<Lce.Content<ManageDevicesUiState>>(contentState)
            assertEquals(3, contentState.value.devices.size)
        }
    }

    @Test
    fun `fetchDevices should update state to Error on failure`() = runTest {
        val error = GetDeviceListError.Unknown(RuntimeException("Network failed"))
        coEvery { mockDeviceRepository.deviceList(testAccountNumber) } returns error.left()

        viewModel.uiState.test {
            val errorState = awaitItem()
            assertIs<Lce.Error<GetDeviceListError>>(errorState)
            assertEquals(error, errorState.error)
        }
    }

    @Test
    fun `the logged in device should appear first in the list`() = runTest {
        viewModel.uiState.test {
            val contentState = awaitItem()
            assertIs<Lce.Content<ManageDevicesUiState>>(contentState)

            val devices = contentState.value.devices
            assertEquals(testDeviceId2, devices[0].device.id)
            assert(devices[0].isCurrentDevice)
            assertFalse(devices[1].isCurrentDevice)
            assertFalse(devices[2].isCurrentDevice)
        }
    }
}
