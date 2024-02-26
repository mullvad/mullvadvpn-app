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
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class NewDeviceUseNotificationCaseTest {

    private val deviceName = "Frank Zebra"
    private val deviceState =
        MutableStateFlow<DeviceState>(
            DeviceState.LoggedIn(
                accountAndDevice = AccountAndDevice("", Device("", deviceName, byteArrayOf(), ""))
            )
        )
    private lateinit var newDeviceNotificationUseCase: NewDeviceNotificationUseCase

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val mockDeviceRepository: DeviceRepository = mockk()
        every { mockDeviceRepository.deviceState } returns deviceState
        newDeviceNotificationUseCase =
            NewDeviceNotificationUseCase(deviceRepository = mockDeviceRepository)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `notifications should be empty by default`() = runTest {
        // Arrange, Act, Assert
        newDeviceNotificationUseCase.notifications().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `when newDeviceCreated is called notifications should emit NewDevice notification containing device name`() =
        runTest {
            newDeviceNotificationUseCase.notifications().test {
                // Arrange, Act
                awaitItem()
                newDeviceNotificationUseCase.newDeviceCreated()

                // Assert
                assertEquals(awaitItem(), listOf(InAppNotification.NewDevice(deviceName)))
            }
        }

    @Test
    fun `clearNewDeviceCreatedNotification should clear notifications`() = runTest {
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
