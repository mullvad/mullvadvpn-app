package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import android.content.Intent
import arrow.core.Either
import co.touchlab.kermit.Logger
import java.time.Clock
import java.time.Instant
import java.time.temporal.ChronoUnit
import kotlin.time.Duration
import kotlin.time.toJavaDuration
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.common.constant.KEY_PAUSE_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_PAUSE_DURATION_SECONDS
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

@Suppress("TooManyFunctions")
class TunnelPauseRepository(
    private val context: Context,
    private val connectionProxy: ConnectionProxy,
    private val deviceRepository: DeviceRepository,
    private val userPreferencesRepository: UserPreferencesRepository,
    private val alarm: PauseAlarm,
    private val clock: Clock = Clock.systemUTC(),
) {
    private val _pausedUntil = MutableStateFlow<Instant?>(null)
    private val alarms = MutableStateFlow(0L)
    private val cancelRequests = MutableStateFlow(0L)

    // Set while a pause is active, holds the time at which the tunnel will be reconnected.
    val pausedUntil: StateFlow<Instant?> = _pausedUntil.asStateFlow()

    // The pause itself runs in MullvadVpnService, so that it keeps running when the app is closed.
    fun requestPause(duration: Duration): Either<Throwable, Unit> =
        Either.catch {
                val intent =
                    Intent(KEY_PAUSE_ACTION)
                        .setClassName(context.packageName, VPN_SERVICE_CLASS)
                        .putExtra(KEY_PAUSE_DURATION_SECONDS, duration.inWholeSeconds)
                context.startService(intent)
                Unit
            }
            .onLeft { Logger.e("Unable to start pause", it) }

    // Called when the resume alarm goes off. The pause only ends once resumeAt has been reached, so
    // an alarm that was already on its way when a new pause was started is ignored.
    fun onAlarm() {
        alarms.update { it + 1 }
    }

    // Ends the pause and leaves the tunnel disconnected.
    fun cancel() {
        cancelRequests.update { it + 1 }
    }

    // Disconnects the tunnel and suspends until the pause is over. The tunnel is reconnected if
    // the timer runs out, but not if the pause ends in any other way. A cancelled coroutine keeps
    // the pause, the alarm then starts the service again which restores it.
    suspend fun pause(duration: Duration): PauseResult {
        val initialConnectionRequests = connectionProxy.connectionRequests.value
        val initialCancelRequests = cancelRequests.value
        val isReplacingPause = _pausedUntil.value != null
        val resumeAt =
            clock.instant().plus(duration.toJavaDuration()).truncatedTo(ChronoUnit.MILLIS)
        Logger.i("Pausing tunnel for $duration")
        // Must be started before disconnecting, so that the service stays in the foreground when
        // the system unbinds from it.
        start(resumeAt)

        val result =
            connectionProxy
                .disconnect(DisconnectReason.USER_INITIATED_PAUSE)
                .fold(
                    { PauseResult.DisconnectFailed(it) },
                    { wasConnected ->
                        // The tunnel was disconnected in some other way before the pause, e.g.
                        // from the tile, so it should not be reconnected later.
                        if (!wasConnected && !isReplacingPause) {
                            PauseResult.NotConnected
                        } else {
                            awaitEnd(
                                resumeAt,
                                initialConnectionRequests,
                                initialCancelRequests,
                                waitForDisconnect = true,
                            )
                        }
                    },
                )
        return finish(resumeAt, result)
    }

    // Continues a pause that was active when the process was killed. Returns null if there is no
    // such pause.
    suspend fun restore(): PauseResult? {
        val initialConnectionRequests = connectionProxy.connectionRequests.value
        val initialCancelRequests = cancelRequests.value
        val resumeAt = userPreferencesRepository.pausedUntil() ?: return null
        Logger.i("Restoring pause")
        start(resumeAt)

        val result =
            awaitEnd(
                resumeAt,
                initialConnectionRequests,
                initialCancelRequests,
                waitForDisconnect = false,
            )
        return finish(resumeAt, result)
    }

    // Alarms do not survive a reboot.
    suspend fun rescheduleAlarm() {
        userPreferencesRepository.pausedUntil()?.let { alarm.schedule(it, ::onAlarm) }
    }

    private suspend fun start(resumeAt: Instant) {
        _pausedUntil.value = resumeAt
        userPreferencesRepository.setPausedUntil(resumeAt)
        alarm.schedule(resumeAt, ::onAlarm)
    }

    private suspend fun finish(resumeAt: Instant, result: PauseResult): PauseResult {
        val finalResult =
            if (result == PauseResult.Resumed) {
                // Like the tile and the notification, leave it to the daemon to handle a missing
                // VPN permission. It ends up in an error state that is shown to the user.
                connectionProxy
                    .connectWithoutPermissionCheck()
                    .fold({ PauseResult.ReconnectFailed(it) }, { PauseResult.Resumed })
            } else {
                result
            }
        // A pause that is left behind would be restored the next time the service starts.
        withContext(NonCancellable) {
            if (_pausedUntil.compareAndSet(resumeAt, null)) {
                alarm.cancel()
                userPreferencesRepository.clearPausedUntil()
            }
        }
        Logger.i("Pause ended: $finalResult")
        return finalResult
    }

    private suspend fun awaitEnd(
        resumeAt: Instant,
        initialConnectionRequests: Long,
        initialCancelRequests: Long,
        waitForDisconnect: Boolean,
    ): PauseResult =
        merge(
                timerExpired(resumeAt),
                connectionRequested(initialConnectionRequests),
                cancelled(initialCancelRequests),
                tunnelStarted(waitForDisconnect),
                loggedOut(),
            )
            .first()

    // Also checked right away, since a restored pause may already be over.
    private fun timerExpired(resumeAt: Instant) =
        alarms.filter { !clock.instant().isBefore(resumeAt) }.take(1).map { PauseResult.Resumed }

    // Capture the counter before disconnecting so requests during the disconnect RPC are retained.
    // A connect request can take the daemon directly from Disconnecting to Connecting.
    private fun connectionRequested(initialConnectionRequests: Long) =
        connectionProxy.connectionRequests
            .filter { it != initialConnectionRequests }
            .take(1)
            .map { PauseResult.ResumedByUser }

    private fun cancelled(initialCancelRequests: Long) =
        cancelRequests.filter { it != initialCancelRequests }.take(1).map { PauseResult.Cancelled }

    // The user (or the system, e.g. always-on VPN) started the tunnel while it was paused. When
    // pausing, wait for the disconnect to go through first, so the state from before the pause is
    // not mistaken for it.
    private fun tunnelStarted(waitForDisconnect: Boolean): Flow<PauseResult> {
        val tunnelState =
            if (waitForDisconnect) {
                connectionProxy.tunnelState.dropWhile { it !is TunnelState.Disconnected }
            } else {
                connectionProxy.tunnelState
            }
        return tunnelState
            .filter { it !is TunnelState.Disconnected }
            .take(1)
            .map { PauseResult.ResumedByUser }
    }

    private fun loggedOut() =
        deviceRepository.deviceState
            .filter { it is DeviceState.LoggedOut || it is DeviceState.Revoked }
            .take(1)
            .map { PauseResult.LoggedOut }
}

sealed interface PauseResult {
    // The timer ran out and the tunnel is reconnecting.
    data object Resumed : PauseResult

    // The tunnel was started before the timer ran out.
    data object ResumedByUser : PauseResult

    // The user ended the pause, the tunnel stays disconnected.
    data object Cancelled : PauseResult

    // The tunnel was already disconnected, so there was nothing to pause.
    data object NotConnected : PauseResult

    // The device was logged out or revoked, the tunnel stays disconnected.
    data object LoggedOut : PauseResult

    data class DisconnectFailed(val error: ConnectError) : PauseResult

    data class ReconnectFailed(val error: ConnectError) : PauseResult
}
