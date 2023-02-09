package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import io.mockk.verifyOrder
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.TestCoroutineDispatcher
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.talpid.util.EventNotifier
import net.mullvad.talpid.util.callbackFlowFromSubscription
import org.junit.After
import org.junit.Before
import org.junit.Test

class DeviceRevokedViewModelTest {

    @MockK
    private lateinit var mockedAccountRepository: AccountRepository

    @MockK
    private lateinit var mockedServiceConnectionManager: ServiceConnectionManager

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    private lateinit var viewModel: DeviceRevokedViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        every { mockedServiceConnectionManager.connectionState } returns serviceConnectionState
        viewModel = DeviceRevokedViewModel(
            mockedServiceConnectionManager,
            mockedAccountRepository,
            TestCoroutineDispatcher()
        )
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testUiStateWhenServiceNotConnected() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value = ServiceConnectionState.Disconnected
            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
        }
    }

    @Test
    fun testUiStateWhenServiceConnectedButNotReady() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value = ServiceConnectionState.ConnectedNotReady(mockk())
            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
        }
    }

    @Test
    fun testUiStateWhenServiceConnectedAndReady() = runBlockingTest {
        // Arrange
        val mockedContainer = mockk<ServiceConnectionContainer>().apply {
            val eventNotifierMock = mockk<EventNotifier<TunnelState>>().apply {
                every { callbackFlowFromSubscription(any()) } returns MutableStateFlow(
                    TunnelState.Connected(mockk(), mockk())
                )
            }
            val mockedConnectionProxy = mockk<ConnectionProxy>().apply {
                every { onUiStateChange } returns eventNotifierMock
            }
            every { connectionProxy } returns mockedConnectionProxy
        }

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
            serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)
            assertEquals(DeviceRevokedUiState.SECURED, awaitItem())
        }
    }

    @Test
    fun testGoToLoginWhenDisconnected() {
        // Arrange
        val mockedContainer = mockk<ServiceConnectionContainer>().also {
            every { it.connectionProxy.state } returns TunnelState.Disconnected
            every { it.connectionProxy.disconnect() } just Runs
            every { mockedAccountRepository.logout() } just Runs
        }
        serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        verify {
            mockedAccountRepository.logout()
        }
    }

    @Test
    fun testGoToLoginWhenConnected() {
        // Arrange
        val mockedContainer = mockk<ServiceConnectionContainer>().also {
            every { it.connectionProxy.state } returns TunnelState.Connected(mockk(), mockk())
            every { it.connectionProxy.disconnect() } just Runs
            every { mockedAccountRepository.logout() } just Runs
        }
        serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        verifyOrder {
            mockedContainer.connectionProxy.disconnect()
            mockedAccountRepository.logout()
        }
    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
    }
}
