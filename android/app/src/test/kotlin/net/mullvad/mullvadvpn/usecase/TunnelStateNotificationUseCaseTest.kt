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
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.InAppNotification
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class TunnelStateNotificationUseCaseTest {

    private val mockConnectionProxy: ConnectionProxy = mockk()

    private lateinit var tunnelStateNotificationUseCase: TunnelStateNotificationUseCase

    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        every { mockConnectionProxy.tunnelState } returns tunnelState

        tunnelStateNotificationUseCase =
            TunnelStateNotificationUseCase(connectionProxy = mockConnectionProxy)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        tunnelStateNotificationUseCase().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `when TunnelState is error use case should emit TunnelStateError notification`() = runTest {
        tunnelStateNotificationUseCase().test {
            // Arrange, Act
            assertEquals(emptyList(), awaitItem())
            val errorState: ErrorState = mockk()
            tunnelState.emit(TunnelState.Error(errorState))

            // Assert
            assertEquals(listOf(InAppNotification.TunnelStateError(errorState)), awaitItem())
        }
    }

    @Test
    fun `when TunnelState is Disconnecting with blocking use case should emit TunnelStateBlocked notification`() =
        runTest {
            tunnelStateNotificationUseCase().test {
                // Arrange, Act
                assertEquals(emptyList(), awaitItem())
                tunnelState.emit(TunnelState.Disconnecting(ActionAfterDisconnect.Block))

                // Assert
                assertEquals(listOf(InAppNotification.TunnelStateBlocked), awaitItem())
            }
        }
}
