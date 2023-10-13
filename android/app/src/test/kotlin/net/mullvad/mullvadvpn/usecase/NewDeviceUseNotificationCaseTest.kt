package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class NewDeviceUseNotificationCaseTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val deviceName = "Frank Zebra"
    private val deviceState =
        MutableStateFlow<DeviceState>(
            DeviceState.LoggedIn(
                accountAndDevice = AccountAndDevice("", Device("", deviceName, byteArrayOf(), ""))
            )
        )
    private lateinit var newDeviceNotificationUseCase: NewDeviceNotificationUseCase

    @Before
    fun setup() {
        MockKAnnotations.init(this)

        val mockDeviceRepository: DeviceRepository = mockk()
        every { mockDeviceRepository.deviceState } returns deviceState
        newDeviceNotificationUseCase =
            NewDeviceNotificationUseCase(deviceRepository = mockDeviceRepository)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `ensure empty by default`() = runTest {
        // Arrange, Act, Assert
        newDeviceNotificationUseCase.notifications().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `ensure NewDevice notification is created and contains device name`() = runTest {
        newDeviceNotificationUseCase.notifications().test {
            // Arrange, Act
            awaitItem()
            newDeviceNotificationUseCase.newDeviceCreated()

            // Assert
            assertEquals(awaitItem(), listOf(InAppNotification.NewDevice(deviceName)))
        }
    }

    @Test
    fun `ensure NewDevice notification is cleared`() = runTest {
        newDeviceNotificationUseCase.notifications().test {
            // Arrange, Act
            awaitItem()
            newDeviceNotificationUseCase.newDeviceCreated()
            awaitItem()
            newDeviceNotificationUseCase.clearNewDeviceCreatedNotification()

            // Assert
            assertEquals(awaitItem(), emptyList())
        }
    }
}
