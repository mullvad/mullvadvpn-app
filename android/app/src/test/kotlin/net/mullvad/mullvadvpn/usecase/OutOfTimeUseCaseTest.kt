package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AuthFailedError
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

@ExperimentalCoroutinesApi
class OutOfTimeUseCaseTest {
    private val mockAccountRepository: AccountRepository = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    private lateinit var events: Channel<TunnelState>
    private lateinit var expiry: MutableStateFlow<AccountData?>

    private val dispatcher = StandardTestDispatcher()
    private val scope = TestScope(dispatcher)

    private lateinit var outOfTimeUseCase: OutOfTimeUseCase

    @BeforeEach
    fun setup() {
        events = Channel()
        expiry = MutableStateFlow(null)
        every { mockAccountRepository.accountData } returns expiry
        every { mockConnectionProxy.tunnelState } returns events.consumeAsFlow()

        Dispatchers.setMain(dispatcher)

        outOfTimeUseCase =
            OutOfTimeUseCase(mockConnectionProxy, mockAccountRepository, scope.backgroundScope)
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
            val errorStateCause = ErrorStateCause.AuthFailed(AuthFailedError.ExpiredAccount)
            val tunnelStateError = TunnelState.Error(ErrorState(errorStateCause, true))

            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                events.send(tunnelStateError)
                assertEquals(true, awaitItem())
            }
        }

    @Test
    fun `tunnel is connected should emit false`() =
        scope.runTest {
            // Arrange
            val expiredAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().plusDays(1))
            val tunnelStateChanges =
                listOf(
                    TunnelState.Disconnected(),
                    TunnelState.Connected(mockk(), null, emptyList()),
                    TunnelState.Connecting(null, null, emptyList()),
                    TunnelState.Disconnecting(mockk()),
                    TunnelState.Error(ErrorState(ErrorStateCause.StartTunnelError, false)),
                )

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
            val expiredAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().minusDays(1))
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
            val notExpiredAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().plusDays(1))

            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                assertEquals(null, awaitItem())
                expiry.emit(notExpiredAccountExpiry)
                assertEquals(false, awaitItem())
            }
        }

    @Test
    fun `account that expires without new expiry event should emit true`() =
        scope.runTest {
            // Arrange
            val expiredAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().plusSeconds(100))
            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                // Initial event
                assertEquals(null, awaitItem())

                expiry.emit(expiredAccountExpiry)
                assertEquals(false, awaitItem())

                // After 50 seconds we should still not emitted out of time
                advanceTimeBy(50.seconds)
                expectNoEvents()

                // After additional 51 seconds we should be out of time since account is now expired
                advanceTimeBy(51.seconds)
                assertEquals(true, expectMostRecentItem())
            }
        }

    @Test
    fun `account that is about to expire but is refilled should emit false`() =
        scope.runTest {
            // Arrange
            val initialAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().plusSeconds(100))
            val updatedExpiry =
                AccountData(mockk(relaxed = true), initialAccountExpiry.expiryDate.plusDays(30))

            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                // Initial event
                assertEquals(null, awaitItem())

                expiry.emit(initialAccountExpiry)
                assertEquals(false, awaitItem())
                advanceTimeBy(90.seconds)
                expectNoEvents()

                // User fills up with more time 10 seconds before expiry.
                expiry.emit(updatedExpiry)
                advanceTimeBy(29.days)
                expectNoEvents()

                advanceTimeBy(2.days)
                assertEquals(true, expectMostRecentItem())
                expectNoEvents()
            }
        }

    @Test
    fun `expired account that is refilled should emit false`() =
        scope.runTest {
            // Arrange
            val initialAccountExpiry =
                AccountData(mockk(relaxed = true), ZonedDateTime.now().plusSeconds(100))
            val updatedExpiry =
                AccountData(mockk(relaxed = true), initialAccountExpiry.expiryDate.plusDays(30))
            // Act, Assert
            outOfTimeUseCase.isOutOfTime.test {
                // Initial event
                assertEquals(null, awaitItem())

                expiry.emit(initialAccountExpiry)
                assertEquals(false, awaitItem())

                // After 100 seconds we expire
                advanceTimeBy(100.seconds)
                assertEquals(true, expectMostRecentItem())
                expectNoEvents()

                // We then fill up our account and should no longer be out of time
                expiry.emit(updatedExpiry)
                assertEquals(false, awaitItem())
                expectNoEvents()

                // Advance the time to before the updated expiry
                advanceTimeBy(29.days + 59.minutes)
                expectNoEvents()

                // Advance the time to the updated expiry
                advanceTimeBy(30.days + 2.minutes)
                assertEquals(true, expectMostRecentItem())
                expectNoEvents()
            }
        }
}
