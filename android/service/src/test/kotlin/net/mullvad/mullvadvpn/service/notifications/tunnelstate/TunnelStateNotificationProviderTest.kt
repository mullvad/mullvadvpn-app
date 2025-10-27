package net.mullvad.mullvadvpn.service.notifications.tunnelstate

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.every
import io.mockk.mockk
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationId
import net.mullvad.mullvadvpn.lib.model.NotificationTunnelState
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.PrepareVpnUseCase
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.repository.UserPreferences
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class TunnelStateNotificationProviderTest {

    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var prepareVpnUseCase: PrepareVpnUseCase
    private lateinit var deviceRepository: DeviceRepository
    private lateinit var userPreferencesRepository: UserPreferencesRepository

    private lateinit var tunnelStateFlow: MutableStateFlow<TunnelState>
    private lateinit var deviceStateFlow: MutableStateFlow<DeviceState>
    private lateinit var preferencesFlow: MutableStateFlow<UserPreferences>

    private val testNotificationId = NotificationId(2)
    private val testChannelId = NotificationChannelId("test_channel")

    private lateinit var provider: TunnelStateNotificationProvider

    private val testAccountNumber = AccountNumber("1234567890123456")
    private val testDeviceId1 = DeviceId.fromString("12345678-1234-5678-1234-567812345678")

    private val testDevice =
        Device(
            id = testDeviceId1,
            name = "Device 1",
            creationDate = ZonedDateTime.now().minusSeconds(100),
        )

    @OptIn(ExperimentalCoroutinesApi::class)
    @BeforeEach
    fun setup() {
        // Initialize mocks and default behaviors
        connectionProxy = mockk()
        prepareVpnUseCase = mockk()
        deviceRepository = mockk()
        userPreferencesRepository = mockk()
        val userPreferences = mockk<UserPreferences>()
        every { userPreferences.showLocationInSystemNotification } returns false

        // Setup controllable StateFlows
        tunnelStateFlow = MutableStateFlow(TunnelState.Disconnected(null))
        deviceStateFlow = MutableStateFlow(DeviceState.LoggedIn(testAccountNumber, testDevice))
        preferencesFlow = MutableStateFlow(userPreferences)

        // Link flows to repository mocks
        every { connectionProxy.tunnelState } returns tunnelStateFlow
        every { deviceRepository.deviceState } returns deviceStateFlow
        every { userPreferencesRepository.preferencesFlow() } returns preferencesFlow

        // Default behavior for prepareVpnUseCase
        every { prepareVpnUseCase.invoke() } returns Prepared.right()

        // Initialize the class under test
        provider =
            TunnelStateNotificationProvider(
                connectionProxy = connectionProxy,
                vpnPermissionRepository = prepareVpnUseCase,
                deviceRepository = deviceRepository,
                preferences = userPreferencesRepository,
                channelId = testChannelId,
                scope = CoroutineScope(UnconfinedTestDispatcher()),
            )
    }

    @Test
    fun `should emit cancel notification when device state is logged out and tunnel state is disconnected`() =
        runTest {
            provider.notifications.test {
                val initialItem = awaitItem()
                assertTrue(initialItem is NotificationUpdate.Notify)
                assertTrue(initialItem.value.state is NotificationTunnelState.Disconnected)

                // When device state changes to LoggedOut
                deviceStateFlow.value = DeviceState.LoggedOut

                // Then a Cancel notification update is emitted
                val cancelItem = awaitItem()
                assertTrue(cancelItem is NotificationUpdate.Cancel)
            }
        }

    @Test
    fun `should emit connected notification when tunnel state is connected`() = runTest {
        provider.notifications.test {
            awaitItem() // Skip initial emission

            // When tunnel state becomes Connected
            tunnelStateFlow.value = TunnelState.Connected(mockk(), mockk(), emptyList())

            // Then a Connected notification update is emitted
            val update = awaitItem()
            assertTrue(update is NotificationUpdate.Notify)
            assertTrue(update.value.state is NotificationTunnelState.Connected)
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun `should emit disconnected with prepare error when VPN is not prepared`() = runTest {
        // Given VPN is not prepared
        val prepareError = PrepareError.OtherAlwaysOnApp("OtherVPN")
        every { prepareVpnUseCase.invoke() } returns prepareError.left()

        provider =
            TunnelStateNotificationProvider(
                connectionProxy = connectionProxy,
                vpnPermissionRepository = prepareVpnUseCase,
                deviceRepository = deviceRepository,
                preferences = userPreferencesRepository,
                channelId = testChannelId,
                scope = CoroutineScope(UnconfinedTestDispatcher()),
            )

        provider.notifications.test {
            val item = awaitItem() as NotificationUpdate.Notify<Notification.Tunnel>

            // Then a Disconnected notification update is emitted with the pepare error
            assertEquals(testNotificationId, item.notificationId)
            val expectedState = NotificationTunnelState.Disconnected(prepareError)
            assertEquals(expectedState, item.value.state)
        }
    }

    @Test
    fun `should re-emit current notification if the show location in notification preference is changed`() =
        runTest {
            val location =
                GeoIpLocation(
                    country = "USA",
                    latitude = 40.7128,
                    longitude = -74.0060,
                    ipv4 = null,
                    ipv6 = null,
                    city = null,
                    hostname = null,
                    entryHostname = null,
                )

            provider.notifications.test {
                awaitItem() // Skip initial

                // When tunnel state becomes Connected
                tunnelStateFlow.value =
                    TunnelState.Connected(mockk(), location = location, emptyList())

                // Then a Connected notification update is emitted
                val update = awaitItem()
                assertTrue(update is NotificationUpdate.Notify)
                assertTrue(update.value.state is NotificationTunnelState.Connected)
                // but the location is null because showLocationInSystemNotification is false
                assertEquals(
                    (update.value.state as NotificationTunnelState.Connected).location,
                    null,
                )

                // When show location in notification preference is changed to true
                preferencesFlow.value =
                    mockk<UserPreferences>().also {
                        every { it.showLocationInSystemNotification } returns true
                    }

                // Then the notification should be re-emitted
                val update2 = awaitItem()
                assertTrue(update2 is NotificationUpdate.Notify)
                assertTrue(update2.value.state is NotificationTunnelState.Connected)
                // And the location is now present in the notification
                assertEquals(
                    (update2.value.state as NotificationTunnelState.Connected).location,
                    location,
                )
            }
        }
}
