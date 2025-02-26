package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class NewDeviceNotificationUseCaseTest {

    private val deviceName = "Frank Zebra"
    private val deviceState =
        MutableStateFlow<DeviceState?>(
            DeviceState.LoggedIn(
                AccountNumber("1234123412341234"),
                Device(
                    id = DeviceId.fromString(UUID),
                    name = deviceName,
                    creationDate = ZonedDateTime.now(),
                ),
            )
        )
    private val isNewDeviceState = MutableStateFlow(false)

    private lateinit var newDeviceNotificationUseCase: NewDeviceNotificationUseCase

    @MockK lateinit var mockDeviceRepository: DeviceRepository

    @MockK lateinit var mockNewDeviceRepository: NewDeviceRepository

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        every { mockNewDeviceRepository.isNewDevice } returns isNewDeviceState
        every { mockDeviceRepository.deviceState } returns deviceState

        newDeviceNotificationUseCase =
            NewDeviceNotificationUseCase(
                newDeviceRepository = mockNewDeviceRepository,
                deviceRepository = mockDeviceRepository,
            )
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        newDeviceNotificationUseCase().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `when a new device has been created a device notification should be emitted`() = runTest {
        newDeviceNotificationUseCase().test {
            // Arrange, Act
            awaitItem()
            newDeviceNotificationUseCase.invoke()
            isNewDeviceState.value = true

            // Assert
            assertEquals(awaitItem(), listOf(InAppNotification.NewDevice(deviceName)))
        }
    }

    @Test
    fun `when a device is unmarked as new the device notification should be cleared`() = runTest {
        // Arrange
        isNewDeviceState.value = true

        // Act
        newDeviceNotificationUseCase().test {
            awaitItem()
            isNewDeviceState.value = false

            // Assert
            assertEquals(awaitItem(), emptyList())
        }
    }
}
