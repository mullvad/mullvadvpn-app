package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.VpnPermissionRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.util.toInAddress
import net.mullvad.mullvadvpn.util.toOutAddress
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ConnectViewModelTest {

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var viewModel: ConnectViewModel

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)
    private val accountExpiryState = MutableStateFlow<AccountData?>(null)
    private val device = MutableStateFlow<DeviceState?>(null)
    private val notifications = MutableStateFlow<List<InAppNotification>>(emptyList())

    // Service connections
    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockLocation: GeoIpLocation = mockk(relaxed = true)

    // Account Repository
    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)

    // Device Repository
    private val mockDeviceRepository: DeviceRepository = mockk()

    // In App Notifications
    private val mockInAppNotificationController: InAppNotificationController = mockk()

    // Select location use case
    private val mockSelectedLocationTitleUseCase: SelectedLocationTitleUseCase = mockk()

    // Payment use case
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    // Flows
    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())
    private val selectedRelayItemFlow = MutableStateFlow<String?>(null)

    // Out Of Time Use Case
    private val outOfTimeUseCase: OutOfTimeUseCase = mockk()
    private val outOfTimeViewFlow = MutableStateFlow(false)

    // Last known location
    private val mockLastKnownLocationUseCase: LastKnownLocationUseCase = mockk()

    // VpnPermissionRepository
    private val mockVpnPermissionRepository: VpnPermissionRepository = mockk(relaxed = true)

    @BeforeEach
    fun setup() {
        mockkStatic(TUNNEL_ENDPOINT_EXTENSIONS)
        mockkStatic(GEO_IP_LOCATIONS_EXTENSIONS)

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState

        every { mockAccountRepository.accountData } returns accountExpiryState

        every { mockDeviceRepository.deviceState } returns device

        coEvery { mockDeviceRepository.updateDevice() } just Runs

        every { mockInAppNotificationController.notifications } returns notifications

        every { mockConnectionProxy.tunnelState } returns tunnelState

        every { mockLastKnownLocationUseCase.lastKnownDisconnectedLocation } returns flowOf(null)

        every { mockLocation.country } returns "dummy country"

        // Flows
        every { mockSelectedLocationTitleUseCase() } returns selectedRelayItemFlow

        every { outOfTimeUseCase.isOutOfTime } returns outOfTimeViewFlow
        viewModel =
            ConnectViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                inAppNotificationController = mockInAppNotificationController,
                newDeviceRepository = mockk(),
                outOfTimeUseCase = outOfTimeUseCase,
                paymentUseCase = mockPaymentUseCase,
                selectedLocationTitleUseCase = mockSelectedLocationTitleUseCase,
                connectionProxy = mockConnectionProxy,
                lastKnownLocationUseCase = mockLastKnownLocationUseCase,
                vpnPermissionRepository = mockVpnPermissionRepository,
                isPlayBuild = false
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `uiState should emit initial state by default`() = runTest {
        viewModel.uiState.test { assertEquals(ConnectUiState.INITIAL, awaitItem()) }
    }

    @Test
    fun `given change in tunnelRealState uiState should emit new tunnelRealState`() = runTest {
        val tunnelRealStateTestItem = TunnelState.Connected(mockk(relaxed = true), null)

        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            tunnelState.emit(tunnelRealStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelRealStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun `given change in tunnelState uiState should emit new tunnelState`() = runTest {
        // Arrange
        val tunnelEndpoint: TunnelEndpoint = mockk()
        val location: GeoIpLocation = mockk()
        val tunnelStateTestItem = TunnelState.Connected(tunnelEndpoint, location)
        every { tunnelEndpoint.toInAddress() } returns mockk(relaxed = true)
        every { location.toOutAddress() } returns "1.1.1.1"
        every { location.hostname } returns "hostname"

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            tunnelState.emit(tunnelStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun `given RelayListUseCase returns new selectedRelayItem uiState should emit new selectedRelayItem`() =
        runTest {
            val selectedRelayItemTitle = "Item"
            selectedRelayItemFlow.value = selectedRelayItemTitle

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                val result = awaitItem()
                assertEquals(selectedRelayItemTitle, result.selectedRelayItemTitle)
            }
        }

    @Test
    fun `given new location in tunnel state uiState should emit new location`() = runTest {
        val locationTestItem =
            GeoIpLocation(
                ipv4 = mockk(relaxed = true),
                ipv6 = mockk(relaxed = true),
                country = "Sweden",
                city = "Gothenburg",
                hostname = "Host",
                latitude = 57.7065,
                longitude = 11.967
            )

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            tunnelState.emit(TunnelState.Disconnected(null))

            // Start of with no location
            assertNull(awaitItem().location)

            // After updated we show latest
            tunnelState.emit(TunnelState.Disconnected(locationTestItem))
            assertEquals(locationTestItem, awaitItem().location)
        }
    }

    @Test
    fun `initial state should not include any location`() =
        // Arrange
        runTest {
            val locationTestItem = null

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                expectNoEvents()
                val result = awaitItem()
                assertEquals(locationTestItem, result.location)
            }
        }

    @Test
    fun `onDisconnectClick should invoke disconnect on ConnectionProxy`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()

        // Act
        viewModel.onDisconnectClick()

        // Assert
        coVerify { mockConnectionProxy.disconnect() }
    }

    @Test
    fun `onReconnectClick should invoke reconnect on ConnectionProxy`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.reconnect() } returns true.right()

        // Act
        viewModel.onReconnectClick()

        // Assert
        coVerify { mockConnectionProxy.reconnect() }
    }

    @Test
    fun `onConnectClick should invoke connect on ConnectionProxy`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.connect() } returns true.right()

        // Act
        viewModel.onConnectClick()

        // Asser
        coVerify { mockConnectionProxy.connect() }
    }

    @Test
    fun `onCancelClick should invoke disconnect on ConnectionProxy`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()

        // Act
        viewModel.onCancelClick()

        // Assert
        coVerify { mockConnectionProxy.disconnect() }
    }

    @Test
    fun `given InAppNotificationController returns TunnelStateError notification uiState should emit notification`() =
        runTest {
            // Arrange
            val mockErrorState: ErrorState = mockk()
            val expectedConnectNotificationState =
                InAppNotification.TunnelStateError(mockErrorState)
            val tunnelStateError = TunnelState.Error(mockErrorState)
            notifications.value = listOf(expectedConnectNotificationState)

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                tunnelState.emit(tunnelStateError)
                val result = awaitItem()
                assertEquals(expectedConnectNotificationState, result.inAppNotification)
            }
        }

    @Test
    fun `onShowAccountClick call should result in uiSideEffect emitting OpenAccountManagementPageInBrowser`() =
        runTest {
            // Arrange
            val mockToken = WebsiteAuthToken.fromString("154c4cc94810fddac78398662b7fa0c7")
            coEvery { mockAccountRepository.getWebsiteAuthToken() } returns mockToken

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.onManageAccountClick()
                val action = awaitItem()
                assertIs<ConnectViewModel.UiSideEffect.OpenAccountManagementPageInBrowser>(action)
                assertEquals(mockToken, action.token)
            }
        }

    @Test
    fun `given OutOfTimeUseCase returns true uiSideEffect should emit OutOfTime`() = runTest {
        // Arrange
        val deferred = async { viewModel.uiSideEffect.first() }

        // Act
        viewModel.uiState.test {
            awaitItem()
            outOfTimeViewFlow.value = true
            awaitItem()
        }

        // Assert
        assertIs<ConnectViewModel.UiSideEffect.OutOfTime>(deferred.await())
    }

    companion object {
        private const val TUNNEL_ENDPOINT_EXTENSIONS =
            "net.mullvad.mullvadvpn.util.TunnelEndpointExtensionsKt"
        private const val GEO_IP_LOCATIONS_EXTENSIONS =
            "net.mullvad.mullvadvpn.util.GeoIpLocationExtensionsKt"
    }
}
