package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.days
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class OutOfTimeUseCaseTest {
    private val mockAccountRepository: AccountRepository = mockk()
    private val mockMessageHandler: MessageHandler = mockk()

    private lateinit var events: Channel<Event.TunnelStateChange>
    private lateinit var expiry: MutableStateFlow<AccountExpiry>

    private val dispatcher = StandardTestDispatcher()
    private val scope = TestScope(dispatcher)

    private lateinit var outOfTimeUseCase: OutOfTimeUseCase

    @BeforeEach
    fun setup() {
        events = Channel()
        expiry = MutableStateFlow(AccountExpiry.Missing)
        every { mockAccountRepository.accountExpiryState } returns expiry
        every { mockMessageHandler.events<Event.TunnelStateChange>() } returns
            events.receiveAsFlow()

        Dispatchers.setMain(dispatcher)

        outOfTimeUseCase =
            OutOfTimeUseCase(mockAccountRepository, mockMessageHandler, scope.backgroundScope)
    }

    @AfterEach
    fun teardown() {
        Dispatchers.resetMain()
        unmockkAll()
    }

    @Test
    fun `no events should result in no expiry`() =
        scope.runTest {
            // Arrange
            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test { assertEquals(null, awaitItem()) }
        }

    @Test
    fun `tunnel is blocking because out of time should emit true`() =
        scope.runTest {
            // Arrange
            // Act, Assert
            val errorStateCause = ErrorStateCause.AuthFailed("[EXPIRED_ACCOUNT]")
            val tunnelStateError = TunnelState.Error(ErrorState(errorStateCause, true))
            val errorChange = Event.TunnelStateChange(tunnelStateError)

            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                events.send(errorChange)
                assertEquals(true, awaitItem())
            }
        }

    @Test
    fun `tunnel is connected should emit false`() =
        scope.runTest {
            // Arrange
            val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().plusDays(1))
            val tunnelStateChanges =
                listOf(
                        TunnelState.Disconnected(),
                        TunnelState.Connected(mockk(), null),
                        TunnelState.Connecting(null, null),
                        TunnelState.Disconnecting(mockk()),
                        TunnelState.Error(ErrorState(ErrorStateCause.StartTunnelError, false)),
                    )
                    .map(Event::TunnelStateChange)

            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                events.send(tunnelStateChanges.first())
                expiry.emit(expiredAccountExpiry)
                assertEquals(false, awaitItem())

                tunnelStateChanges.forEach { events.send(it) }

                // Should not emit again
                expectNoEvents()
            }
        }

    @Test
    fun `account expiry that has expired should emit true`() =
        scope.runTest {
            // Arrange
            val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().minusDays(1))
            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                expiry.emit(expiredAccountExpiry)
                assertEquals(true, awaitItem())
            }
        }

    @Test
    fun `account expiry that has not expired should emit false`() =
        scope.runTest {
            // Arrange
            val notExpiredAccountExpiry = AccountExpiry.Available(DateTime.now().plusDays(1))

            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                expiry.emit(notExpiredAccountExpiry)
                assertEquals(false, awaitItem())
            }
        }

    @Test
    fun `account that expires without new expiry event should emit true`() =
        runTest(dispatcher) {
            // Arrange
            val expiredAccountExpiry = AccountExpiry.Available(DateTime.now().plusSeconds(100))
            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                // Initial event
                assertEquals(null, awaitItem())

                expiry.emit(expiredAccountExpiry)
                assertEquals(false, awaitItem())

                // After 50 seconds we should still not emitted out of time
                advanceTimeBy(50_000)
                expectNoEvents()

                // After additional 50 seconds we should be out of time since account is now expired
                advanceTimeBy(50_000)
                assertEquals(true, awaitItem())
            }
        }

    @Test
    fun `account that is about to expire but is refilled should emit false`() = runTest {
        // Arrange
        val initialAccountExpiry = AccountExpiry.Available(DateTime.now().plusSeconds(100))
        val updatedExpiry =
            AccountExpiry.Available(initialAccountExpiry.expiryDateTime.plusDays(30))
        // Act, Assert
        outOfTimeUseCase.isOutOfTime.test {
            // Initial event
            assertEquals(null, awaitItem())

            expiry.emit(initialAccountExpiry)
            assertEquals(false, awaitItem())
            advanceTimeBy(90_000)
            expectNoEvents()
            expiry.emit(updatedExpiry)
            advanceTimeBy(1.days)
            expectNoEvents()
            advanceTimeBy(29.days)
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }

    @Test
    fun `expired account that is refilled should emit false`() = runTest {
        // Arrange
        val initialAccountExpiry = AccountExpiry.Available(DateTime.now().plusSeconds(100))
        val updatedExpiry =
            AccountExpiry.Available(initialAccountExpiry.expiryDateTime.plusDays(30))
        // Act, Assert
        outOfTimeUseCase.isOutOfTime.test {
            // Initial event
            assertEquals(null, awaitItem())

            expiry.emit(initialAccountExpiry)
            assertEquals(false, awaitItem())

            // After 100 seconds we expire
            advanceTimeBy(100_000)
            assertEquals(true, awaitItem())
            expectNoEvents()

            // We then fill up our account and should no longer be out of time
            expiry.emit(updatedExpiry)
            assertEquals(false, awaitItem())
            expectNoEvents()

            // Advance the time to the updated expiry
            advanceTimeBy(30.days)
            assertEquals(true, awaitItem())
            expectNoEvents()
        }
    }
}
