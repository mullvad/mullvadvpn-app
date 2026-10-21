package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import android.content.Intent
import arrow.core.Either
import co.touchlab.kermit.Logger
import java.time.Clock
import java.time.Instant
import kotlin.time.Duration
import kotlin.time.toJavaDuration
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.take
import net.mullvad.mullvadvpn.lib.common.constant.KEY_PAUSE_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_PAUSE_DURATION_SECONDS
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.TunnelState

class TunnelPauseRepository(
    private val context: Context,
    private val connectionProxy: ConnectionProxy,
    private val deviceRepository: DeviceRepository,
    private val timer: PauseTimer,
    private val clock: Clock = Clock.systemUTC(),
) {
    private val _pausedUntil = MutableStateFlow<Instant?>(null)

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

    // Disconnects the tunnel and suspends until the pause is over. The tunnel is reconnected if
    // the timer runs out, but not if the pause ends in any other way.
    suspend fun pause(duration: Duration): PauseResult {
        val initialConnectionRequests = connectionProxy.connectionRequests.value
        val resumeAt = clock.instant().plus(duration.toJavaDuration())
        // Must be set before disconnecting, so that the service stays in the foreground when the
        // system unbinds from it.
        _pausedUntil.value = resumeAt
        Logger.i("Pausing tunnel for $duration")
        try {
            connectionProxy.disconnect(DisconnectReason.USER_INITIATED_PAUSE).onLeft {
                return PauseResult.DisconnectFailed(it)
            }

            val result =
                merge(
                        timerExpired(duration),
                        connectionRequested(initialConnectionRequests),
                        tunnelStarted(),
                        loggedOut(),
                    )
                    .first()
            Logger.i("Pause ended: $result")

            return if (result == PauseResult.Resumed) {
                connectionProxy
                    .connectWithoutPermissionCheck()
                    .fold({ PauseResult.ReconnectFailed(it) }, { PauseResult.Resumed })
            } else {
                result
            }
        } finally {
            _pausedUntil.compareAndSet(resumeAt, null)
        }
    }

    private fun timerExpired(duration: Duration) = flow {
        timer.await(duration)
        emit(PauseResult.Resumed)
    }

    // Capture the counter before disconnecting so requests during the disconnect RPC are retained.
    // A connect request can take the daemon directly from Disconnecting to Connecting.
    private fun connectionRequested(initialConnectionRequests: Long) =
        connectionProxy.connectionRequests
            .filter { it != initialConnectionRequests }
            .take(1)
            .map { PauseResult.ResumedByUser }

    // The user (or the system, e.g. always-on VPN) started the tunnel while it was paused. Wait for
    // the disconnect to go through first, so the state from before the pause is not mistaken for
    // it.
    private fun tunnelStarted() =
        connectionProxy.tunnelState
            .dropWhile { it !is TunnelState.Disconnected }
            .filter { it !is TunnelState.Disconnected }
            .take(1)
            .map { PauseResult.ResumedByUser }

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

    // The device was logged out or revoked, the tunnel stays disconnected.
    data object LoggedOut : PauseResult

    data class DisconnectFailed(val error: ConnectError) : PauseResult

    data class ReconnectFailed(val error: ConnectError) : PauseResult
}
