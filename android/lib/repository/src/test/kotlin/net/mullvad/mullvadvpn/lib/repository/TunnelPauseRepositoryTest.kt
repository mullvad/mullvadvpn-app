package net.mullvad.mullvadvpn.lib.repository

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import kotlin.time.Duration.Companion.minutes
import kotlin.time.toJavaDuration
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.currentTime
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.TunnelState
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

@OptIn(ExperimentalCoroutinesApi::class)
class TunnelPauseRepositoryTest {

    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()

    private val connectionRequests = MutableStateFlow(0L)
    private val tunnelState =
        MutableStateFlow<TunnelState>(
            TunnelState.Connected(
                endpoint = mockk(),
                location = null,
                featureIndicators = emptyList(),
            )
        )
    private val deviceState =
        MutableStateFlow<DeviceState?>(
            DeviceState.LoggedIn(
                accountNumber = AccountNumber("1234123412341234"),
                device = mockk(),
            )
        )

    // Uses the virtual time of the test scheduler.
    private val timer = PauseTimer { delay(it) }

    private lateinit var tunnelPauseRepository: TunnelPauseRepository

    @BeforeEach
    fun setUp() {
        every { mockConnectionProxy.connectionRequests } returns connectionRequests
        every { mockConnectionProxy.tunnelState } returns tunnelState
        every { mockDeviceRepository.deviceState } returns deviceState
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } answers
            {
                tunnelState.value = TunnelState.Disconnected()
                true.right()
            }
        coEvery { mockConnectionProxy.connectWithoutPermissionCheck() } returns true.right()

        tunnelPauseRepository =
            TunnelPauseRepository(
                context = mockk(),
                connectionProxy = mockConnectionProxy,
                deviceRepository = mockDeviceRepository,
                timer = timer,
                clock = Clock.fixed(NOW, ZoneOffset.UTC),
            )
    }

    @Test
    fun `pause should disconnect and reconnect when the timer runs out`() = runTest {
        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.Resumed, result)
        assertEquals(PAUSE_DURATION.inWholeMilliseconds, currentTime)
        coVerify(exactly = 1) {
            mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE)
        }
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pausedUntil should be set while paused and cleared afterwards`() = runTest {
        tunnelPauseRepository.pausedUntil.test {
            assertNull(awaitItem())

            // Act
            launch { tunnelPauseRepository.pause(PAUSE_DURATION) }

            // Assert
            assertEquals(NOW.plus(PAUSE_DURATION.toJavaDuration()), awaitItem())
            assertNull(awaitItem())
        }
    }

    @Test
    fun `pause should not reconnect when the tunnel is started during the pause`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        tunnelState.value =
            TunnelState.Connecting(
                endpoint = null,
                location = null,
                featureIndicators = emptyList(),
            )

        // Assert
        assertEquals(PauseResult.ResumedByUser, result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `connecting before teardown completes should cancel the pause timer`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } answers
            {
                tunnelState.value = TunnelState.Disconnecting(ActionAfterDisconnect.Nothing)
                true.right()
            }
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act: a notification or tile can connect without passing through Disconnected.
        connectionRequests.value += 1
        tunnelState.value =
            TunnelState.Connecting(
                endpoint = null,
                location = null,
                featureIndicators = emptyList(),
            )

        // Assert: a later manual disconnect must not be undone by the original timer.
        assertEquals(PauseResult.ResumedByUser, result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        tunnelState.value = TunnelState.Disconnected()
        advanceTimeBy(PAUSE_DURATION)
        advanceUntilIdle()
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `connection requested during the disconnect RPC should end the pause`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } answers
            {
                connectionRequests.value += 1
                true.right()
            }

        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.ResumedByUser, result)
        assertEquals(0L, currentTime)
        assertNull(tunnelPauseRepository.pausedUntil.value)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `connection requested before the pause should not end it`() = runTest {
        // Arrange
        connectionRequests.value += 1

        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.Resumed, result)
        assertEquals(PAUSE_DURATION.inWholeMilliseconds, currentTime)
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `tunnel state from before the disconnect should not end the pause`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } returns
            true.right()

        // Act
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()
        tunnelState.value = TunnelState.Disconnected()
        advanceTimeBy(PAUSE_DURATION / 2)

        // Assert
        assertFalse(result.isCompleted)
        assertEquals(PauseResult.Resumed, result.await())
    }

    @Test
    fun `pause should end without reconnecting when the device is logged out`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        deviceState.value = DeviceState.LoggedOut

        // Assert
        assertEquals(PauseResult.LoggedOut, result.await())
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pause should end without reconnecting when the device is revoked`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        deviceState.value = DeviceState.Revoked

        // Assert
        assertEquals(PauseResult.LoggedOut, result.await())
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pause should return DisconnectFailed when disconnecting fails`() = runTest {
        // Arrange
        val error = ConnectError.Unknown(RuntimeException())
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } returns
            error.left()

        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.DisconnectFailed(error), result)
        assertEquals(0L, currentTime)
        assertNull(tunnelPauseRepository.pausedUntil.value)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pause should return ReconnectFailed when reconnecting fails`() = runTest {
        // Arrange
        val error = ConnectError.Unknown(RuntimeException())
        coEvery { mockConnectionProxy.connectWithoutPermissionCheck() } returns error.left()

        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.ReconnectFailed(error), result)
        assertNull(tunnelPauseRepository.pausedUntil.value)
    }

    @Test
    fun `cancelling a replaced pause should keep pausedUntil of the new pause`() = runTest {
        // Arrange
        val longerDuration = PAUSE_DURATION * 3
        val first = launch { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()
        val second = launch { tunnelPauseRepository.pause(longerDuration) }
        runCurrent()

        // Act
        first.cancel()
        runCurrent()

        // Assert
        assertEquals(
            NOW.plus(longerDuration.toJavaDuration()),
            tunnelPauseRepository.pausedUntil.value,
        )
        second.cancel()
    }

    companion object {
        private val NOW = Instant.parse("2026-01-01T12:00:00Z")
        private val PAUSE_DURATION = 5.minutes
    }
}
