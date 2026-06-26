package net.mullvad.mullvadvpn.lib.pushnotification.tunnelstate

import android.content.Context
import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import de.infix.testBalloon.framework.core.testSuite
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
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
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.repository.UserPreferences

const val VPN_SERVICE_UTILS = "net.mullvad.mullvadvpn.lib.common.util.VpnServiceUtilsKt"
@ExperimentalCoroutinesApi val TunnelStateNotificationProviderTest by testSuite{
    testFixture {
        mockkStatic(VPN_SERVICE_UTILS)

        object {
             val mockContext: Context = mockk()
             val connectionProxy: ConnectionProxy = mockk()
             val deviceRepository: DeviceRepository = mockk()
             val userPreferencesRepository: UserPreferencesRepository = mockk()

            val testAccountNumber = AccountNumber("1234567890123456")
            val testDeviceId1 = DeviceId.fromString("12345678-1234-5678-1234-567812345678")

            val testDevice =
                Device(
                    id = testDeviceId1,
                    name = "Device 1",
                    creationDate = ZonedDateTime.now().minusSeconds(100),
                )
            val tunnelStateFlow = MutableStateFlow<TunnelState>(TunnelState.Disconnected(null))
            val deviceStateFlow = MutableStateFlow<DeviceState>(DeviceState.LoggedIn(testAccountNumber, testDevice))
            val userPreferences = mockk<UserPreferences>()
            val preferencesFlow = MutableStateFlow(userPreferences)

             val testNotificationId = NotificationId(2)
             val testChannelId = NotificationChannelId("test_channel")

             lateinit var provider: TunnelStateNotificationProvider

        }.also {
            // Initialize mocks and default behaviors
            every { it.userPreferences.showLocationInSystemNotification } returns false

            // Setup controllable StateFlows

            // Link flows to repository mocks
            every { it.connectionProxy.tunnelState } returns it.tunnelStateFlow
            every { it.deviceRepository.deviceState } returns it.deviceStateFlow
            every { it.userPreferencesRepository.preferencesFlow() } returns it.preferencesFlow

            // Default behavior for prepareSafe()
            every { it.mockContext.prepareVpnSafe() } returns Prepared.right()

            // Initialize the class under test
            it.provider =
                TunnelStateNotificationProvider(
                    context = it.mockContext,
                    connectionProxy = it.connectionProxy,
                    deviceRepository = it.deviceRepository,
                    preferences = it.userPreferencesRepository,
                    channelId = it.testChannelId,
                    scope = CoroutineScope(UnconfinedTestDispatcher()),
                )
        }
    } asContextForEach {

        test("should emit cancel notification when device state is logged out and tunnel state is disconnected") {
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
        test("should emit connected notification when tunnel state is connected") {
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

        test("should emit disconnected with prepare error when VPN is not prepared") {
            // Given VPN is not prepared
            val prepareError = PrepareError.OtherAlwaysOnApp("OtherVPN")
            every { mockContext.prepareVpnSafe() } returns prepareError.left()

            provider =
                TunnelStateNotificationProvider(
                    context = mockContext,
                    connectionProxy = connectionProxy,
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

        test("should re-emit current notification if the show location in notification preference is changed") {
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
}
