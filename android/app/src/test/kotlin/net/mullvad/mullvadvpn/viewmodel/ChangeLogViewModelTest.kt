package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.impl.annotations.MockK
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.repository.AppChangesRepository
import net.mullvad.mullvadvpn.repository.ChangeLogState
import org.junit.After
import org.junit.Before
import org.junit.Test

class ChangeLogViewModelTest {

    @MockK
    private lateinit var mockedAppChangesRepository: AppChangesRepository

    private val changeLogState =
        MutableStateFlow(ChangeLogState.Unknown)

    private lateinit var viewModel: AppChangesViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        viewModel = AppChangesViewModel(mockedAppChangesRepository)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testUiStateWhenNeedToShowChangeLog() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.changeLogState.test {
            changeLogState.value = ChangeLogState.ShouldShow
            assertEquals(ChangeLogState.ShouldShow, awaitItem())
        }
    }

//    @Test
//    fun testUiStateWhenServiceConnectedButNotReady() = runBlockingTest {
//        // Arrange, Act, Assert
//        viewModel.uiState.test {
//            serviceConnectionState.value = ServiceConnectionState.ConnectedNotReady(mockk())
//            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
//        }
//    }
//
//    @Test
//    fun testUiStateWhenServiceConnectedAndReady() = runBlockingTest {
//        // Arrange
//        val mockedContainer = mockk<ServiceConnectionContainer>().apply {
//            val eventNotifierMock = mockk<EventNotifier<TunnelState>>().apply {
//                every { callbackFlowFromSubscription(any()) } returns MutableStateFlow(
//                    TunnelState.Connected(mockk(), mockk())
//                )
//            }
//            val mockedConnectionProxy = mockk<ConnectionProxy>().apply {
//                every { onUiStateChange } returns eventNotifierMock
//            }
//            every { connectionProxy } returns mockedConnectionProxy
//        }
//
//        // Act, Assert
//        viewModel.uiState.test {
//            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
//            serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)
//            assertEquals(DeviceRevokedUiState.SECURED, awaitItem())
//        }
//    }
//
//    @Test
//    fun testGoToLoginWhenDisconnected() {
//        // Arrange
//        val mockedContainer = mockk<ServiceConnectionContainer>().also {
//            every { it.connectionProxy.state } returns TunnelState.Disconnected
//            every { it.connectionProxy.disconnect() } just Runs
//            every { mockedAccountRepository.logout() } just Runs
//        }
//        serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)
//
//        // Act
//        viewModel.onGoToLoginClicked()
//
//        // Assert
//        verify {
//            mockedAccountRepository.logout()
//        }
//    }
//
//    @Test
//    fun testGoToLoginWhenConnected() {
//        // Arrange
//        val mockedContainer = mockk<ServiceConnectionContainer>().also {
//            every { it.connectionProxy.state } returns TunnelState.Connected(mockk(), mockk())
//            every { it.connectionProxy.disconnect() } just Runs
//            every { mockedAccountRepository.logout() } just Runs
//        }
//        serviceConnectionState.value = ServiceConnectionState.ConnectedReady(mockedContainer)
//
//        // Act
//        viewModel.onGoToLoginClicked()
//
//        // Assert
//        verifyOrder {
//            mockedContainer.connectionProxy.disconnect()
//            mockedAccountRepository.logout()
//        }
//    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
    }
}
