package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.junit.Before
import org.junit.Test

class OutOfTimeUseCaseTest {

    private val mockAccountRepository: AccountRepository = mockk()
    private val mockMessageHandler: MessageHandler = mockk()

    private val events = MutableSharedFlow<Event.TunnelStateChange>()
    private val expiry = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)

    lateinit var outOfTimeUseCase: OutOfTimeUseCase

    @Before
    fun setup() {
        every { mockAccountRepository.accountExpiryState } returns expiry
        every { mockMessageHandler.events<Event.TunnelStateChange>() } returns events
        outOfTimeUseCase = OutOfTimeUseCase(mockAccountRepository, mockMessageHandler)
    }

    @Test
    fun `No events should result in no expiry`() = runTest {
        // Arrange
        // Act, Assert
        outOfTimeUseCase().test { assertEquals(false, awaitItem()) }
    }

    @Test
    fun `Tunnel is blocking because out of time should emit true`() = runTest {
        // Arrange
        // Act, Assert
        val errorStateCause = ErrorStateCause.AuthFailed("[EXPIRED_ACCOUNT]")
        val tunnelStateError = TunnelState.Error(ErrorState(errorStateCause, true))
        val errorChange = Event.TunnelStateChange(tunnelStateError)

        outOfTimeUseCase().test {
            assertEquals(false, awaitItem())
            events.emit(errorChange)
            assertEquals(true, awaitItem())
        }
    }

    @Test
    fun `Account expiry that has expired should emit true`() = runTest {
        // Arrange
        val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().minusDays(1))
        // Act, Assert
        outOfTimeUseCase().test {
            assertEquals(false, awaitItem())
            expiry.emit(expiredAccountExpiry)
            assertEquals(true, awaitItem())
        }
    }

    @Test
    fun `Account expiry that has not expired should emit nothing`() = runTest {
        // Arrange
        val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().plusDays(1))

        // Act, Assert
        outOfTimeUseCase().test {
            assertEquals(false, awaitItem())
            expiry.emit(expiredAccountExpiry)
            expectNoEvents()
        }
    }

    @Test
    fun `Account that expires without new expiry event`() = runTest {
        // Arrange
        val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().plusSeconds(2))

        // Act, Assert
        outOfTimeUseCase().test {
            // Initial event that doesn't change
            assertEquals(false, awaitItem())
            expiry.emit(expiredAccountExpiry)

            assertEquals(true, awaitItem())
        }
    }
}
