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
import java.time.ZoneId
import java.time.ZoneOffset
import kotlin.time.Duration.Companion.minutes
import kotlin.time.toJavaDuration
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

@OptIn(ExperimentalCoroutinesApi::class)
class TunnelPauseRepositoryTest {

    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockUserPreferencesRepository: UserPreferencesRepository = mockk()

    private val connectionRequests = MutableStateFlow(0L)
    private val tunnelState = MutableStateFlow<TunnelState>(connected())
    private val deviceState =
        MutableStateFlow<DeviceState?>(
            DeviceState.LoggedIn(
                accountNumber = AccountNumber("1234123412341234"),
                device = mockk(),
            )
        )

    private val clock = MutableClock(NOW)
    private val alarm = FakePauseAlarm()

    // The pause as it is persisted in the user preferences.
    private var storedPausedUntil: Instant? = null

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
        coEvery { mockUserPreferencesRepository.setPausedUntil(any()) } answers
            {
                storedPausedUntil = firstArg()
            }
        coEvery { mockUserPreferencesRepository.clearPausedUntil() } answers
            {
                storedPausedUntil = null
            }
        coEvery { mockUserPreferencesRepository.pausedUntil() } answers { storedPausedUntil }

        tunnelPauseRepository =
            TunnelPauseRepository(
                context = mockk(),
                connectionProxy = mockConnectionProxy,
                deviceRepository = mockDeviceRepository,
                userPreferencesRepository = mockUserPreferencesRepository,
                alarm = alarm,
                clock = clock,
            )
    }

    @Test
    fun `pause should disconnect and reconnect when the alarm goes off`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()
        assertFalse(result.isCompleted)

        // Act
        expireTimer(RESUME_AT)

        // Assert
        assertEquals(PauseResult.Resumed, result.await())
        coVerify(exactly = 1) {
            mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE)
        }
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pause should persist the resume time and schedule the alarm`() = runTest {
        // Act
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Assert
        assertEquals(RESUME_AT, storedPausedUntil)
        assertEquals(RESUME_AT, alarm.resumeAt)

        expireTimer(RESUME_AT)
        result.await()
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
    }

    @Test
    fun `pausedUntil should be set while paused and cleared afterwards`() = runTest {
        tunnelPauseRepository.pausedUntil.test {
            assertNull(awaitItem())

            // Act
            launch { tunnelPauseRepository.pause(PAUSE_DURATION) }

            // Assert
            assertEquals(RESUME_AT, awaitItem())
            expireTimer(RESUME_AT)
            assertNull(awaitItem())
        }
    }

    @Test
    fun `alarm before the resume time should not end the pause`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act: e.g. an alarm of an earlier pause that was already on its way.
        tunnelPauseRepository.onAlarm()
        runCurrent()

        // Assert
        assertFalse(result.isCompleted)
        expireTimer(RESUME_AT)
        assertEquals(PauseResult.Resumed, result.await())
    }

    @Test
    fun `pause should not reconnect when the tunnel is started during the pause`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        tunnelState.value = connecting()

        // Assert
        assertEquals(PauseResult.ResumedByUser, result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `connecting before teardown completes should end the pause`() = runTest {
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
        tunnelState.value = connecting()

        // Assert: a later manual disconnect must not be undone by the original alarm.
        assertEquals(PauseResult.ResumedByUser, result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        tunnelState.value = TunnelState.Disconnected()
        clock.now = RESUME_AT
        tunnelPauseRepository.onAlarm()
        runCurrent()
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
        assertNull(tunnelPauseRepository.pausedUntil.value)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `connection requested before the pause should not end it`() = runTest {
        // Arrange
        connectionRequests.value += 1

        // Act
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()
        expireTimer(RESUME_AT)

        // Assert
        assertEquals(PauseResult.Resumed, result.await())
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
        runCurrent()

        // Assert
        assertFalse(result.isCompleted)
        expireTimer(RESUME_AT)
        assertEquals(PauseResult.Resumed, result.await())
    }

    @Test
    fun `pause should not reconnect when the tunnel was already disconnected`() = runTest {
        // Arrange
        tunnelState.value = TunnelState.Disconnected()
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } returns
            false.right()

        // Act
        val result = tunnelPauseRepository.pause(PAUSE_DURATION)

        // Assert
        assertEquals(PauseResult.NotConnected, result)
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `cancel should end the pause without reconnecting`() = runTest {
        // Arrange
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        tunnelPauseRepository.cancel()

        // Assert
        assertEquals(PauseResult.Cancelled, result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `cancel before the pause should not end it`() = runTest {
        // Arrange
        tunnelPauseRepository.cancel()

        // Act
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Assert
        assertFalse(result.isCompleted)
        expireTimer(RESUME_AT)
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
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pause should return ReconnectFailed when reconnecting fails`() = runTest {
        // Arrange
        val error = ConnectError.Unknown(RuntimeException())
        coEvery { mockConnectionProxy.connectWithoutPermissionCheck() } returns error.left()
        val result = async { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act
        expireTimer(RESUME_AT)

        // Assert
        assertEquals(PauseResult.ReconnectFailed(error), result.await())
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(storedPausedUntil)
    }

    @Test
    fun `cancelled pause coroutine should keep the pause`() = runTest {
        // Arrange
        val pause = launch { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()

        // Act: e.g. the service is destroyed.
        pause.cancelAndJoin()

        // Assert: the alarm starts the service again, which restores the pause.
        assertEquals(RESUME_AT, tunnelPauseRepository.pausedUntil.value)
        assertEquals(RESUME_AT, storedPausedUntil)
        assertEquals(RESUME_AT, alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `pausing again should replace the running pause`() = runTest {
        // Arrange
        val longerDuration = PAUSE_DURATION * 3
        val longerResumeAt = NOW.plus(longerDuration.toJavaDuration())
        val first = launch { tunnelPauseRepository.pause(PAUSE_DURATION) }
        runCurrent()
        first.cancelAndJoin()
        // The tunnel is still disconnected from the first pause.
        coEvery { mockConnectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE) } returns
            false.right()

        // Act
        val second = async { tunnelPauseRepository.pause(longerDuration) }
        runCurrent()
        expireTimer(RESUME_AT)

        // Assert
        assertFalse(second.isCompleted)
        assertEquals(longerResumeAt, tunnelPauseRepository.pausedUntil.value)
        assertEquals(longerResumeAt, storedPausedUntil)
        assertEquals(longerResumeAt, alarm.resumeAt)
        expireTimer(longerResumeAt)
        assertEquals(PauseResult.Resumed, second.await())
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `restore should do nothing when there is no pause`() = runTest {
        // Act
        val result = tunnelPauseRepository.restore()

        // Assert
        assertNull(result)
        assertNull(tunnelPauseRepository.pausedUntil.value)
        assertNull(alarm.resumeAt)
    }

    @Test
    fun `restore should continue the pause until the alarm goes off`() = runTest {
        // Arrange
        storedPausedUntil = RESUME_AT
        tunnelState.value = TunnelState.Disconnected()

        // Act
        val result = async { tunnelPauseRepository.restore() }
        runCurrent()

        // Assert
        assertFalse(result.isCompleted)
        assertEquals(RESUME_AT, tunnelPauseRepository.pausedUntil.value)
        assertEquals(RESUME_AT, alarm.resumeAt)
        expireTimer(RESUME_AT)
        assertEquals(PauseResult.Resumed, result.await())
        assertNull(storedPausedUntil)
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `restore should reconnect right away when the pause is already over`() = runTest {
        // Arrange
        storedPausedUntil = RESUME_AT
        tunnelState.value = TunnelState.Disconnected()
        clock.now = RESUME_AT.plusSeconds(1)

        // Act
        val result = tunnelPauseRepository.restore()

        // Assert
        assertEquals(PauseResult.Resumed, result)
        assertNull(storedPausedUntil)
        coVerify(exactly = 1) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `restore should end the pause when the tunnel is already running`() = runTest {
        // Arrange
        storedPausedUntil = RESUME_AT

        // Act
        val result = tunnelPauseRepository.restore()

        // Assert
        assertEquals(PauseResult.ResumedByUser, result)
        assertNull(storedPausedUntil)
        assertNull(alarm.resumeAt)
        coVerify(exactly = 0) { mockConnectionProxy.connectWithoutPermissionCheck() }
    }

    @Test
    fun `rescheduleAlarm should schedule the persisted pause`() = runTest {
        // Arrange
        storedPausedUntil = RESUME_AT

        // Act
        tunnelPauseRepository.rescheduleAlarm()

        // Assert
        assertEquals(RESUME_AT, alarm.resumeAt)
    }

    @Test
    fun `rescheduleAlarm should do nothing when there is no pause`() = runTest {
        // Act
        tunnelPauseRepository.rescheduleAlarm()

        // Assert
        assertNull(alarm.resumeAt)
    }

    private fun expireTimer(resumeAt: Instant) {
        clock.now = resumeAt
        alarm.fire()
    }

    private fun connected() =
        TunnelState.Connected(endpoint = mockk(), location = null, featureIndicators = emptyList())

    private fun connecting() =
        TunnelState.Connecting(endpoint = null, location = null, featureIndicators = emptyList())

    private class MutableClock(var now: Instant) : Clock() {
        override fun instant(): Instant = now

        override fun getZone(): ZoneId = ZoneOffset.UTC

        override fun withZone(zone: ZoneId): Clock = this
    }

    private class FakePauseAlarm : PauseAlarm {
        var resumeAt: Instant? = null
            private set

        private var onAlarm: (() -> Unit)? = null

        override fun schedule(resumeAt: Instant, onAlarm: () -> Unit) {
            this.resumeAt = resumeAt
            this.onAlarm = onAlarm
        }

        override fun cancel() {
            resumeAt = null
            onAlarm = null
        }

        fun fire() {
            onAlarm?.invoke()
        }
    }

    companion object {
        private val NOW = Instant.parse("2026-01-01T12:00:00Z")
        private val PAUSE_DURATION = 5.minutes
        private val RESUME_AT = NOW.plus(PAUSE_DURATION.toJavaDuration())
    }
}
