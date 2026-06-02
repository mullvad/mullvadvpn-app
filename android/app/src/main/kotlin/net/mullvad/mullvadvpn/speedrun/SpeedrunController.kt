package net.mullvad.mullvadvpn.speedrun

import android.content.ClipboardManager
import android.content.Context
import android.os.SystemClock
import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.compose.SpeedrunGate
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

enum class SpeedrunPhase {
    NOT_STARTED,
    RUNNING,
    FINISHED,
}

/**
 * Elapsed time that resists tampering: the monotonic delta ([SystemClock.elapsedRealtime]) ignores
 * wall-clock changes (so moving the clock back can't shrink it), while the wall-clock delta
 * survives a reboot (which resets the monotonic clock). Taking the max means neither trick can
 * under-report.
 */
fun tamperResistantElapsed(
    startWallMillis: Long,
    startElapsedMillis: Long,
    nowWallMillis: Long,
    nowElapsedMillis: Long,
): Long =
    maxOf(nowWallMillis - startWallMillis, nowElapsedMillis - startElapsedMillis).coerceAtLeast(0L)

data class SpeedrunUiState(
    val phase: SpeedrunPhase = SpeedrunPhase.NOT_STARTED,
    val startWallMillis: Long = 0L,
    val startElapsedMillis: Long = 0L,
    val finishWallMillis: Long = 0L,
    val finishElapsedMillis: Long = 0L,
    val currentIndex: Int = 0,
    // Live (not persisted) flags used only to gate the Start button.
    val baselineKnown: Boolean = false,
    val baselineClean: Boolean = false,
) {
    val totalTasks: Int = SPEEDRUN_TASKS.size
    val currentTask: SpeedrunTask? = SPEEDRUN_TASKS.getOrNull(currentIndex)
    val finalElapsedMillis: Long =
        tamperResistantElapsed(
            startWallMillis,
            startElapsedMillis,
            finishWallMillis,
            finishElapsedMillis,
        )
}

/**
 * TEMPORARY speed-run engine.
 *
 * Registered as a Koin singleton so detection keeps running regardless of which screen is showing
 * or whether the overlay is currently composed. State is persisted to SharedPreferences, so a
 * mid-run process death resumes the run, and clearing app data (the documented prerequisite) is the
 * intended way to reset for the next team.
 */
class SpeedrunController(
    deviceRepository: DeviceRepository,
    connectionProxy: ConnectionProxy,
    settingsRepository: SettingsRepository,
    context: Context,
) : SpeedrunGate {
    private val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val clipboard = context.getSystemService(ClipboardManager::class.java)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _uiState = MutableStateFlow(loadPersistedState())
    val uiState: StateFlow<SpeedrunUiState> = _uiState.asStateFlow()

    /** Exposed to other modules (login, privacy disclaimer) so they can gate input on the run. */
    override val isRunning: StateFlow<Boolean> =
        _uiState
            .map { it.phase == SpeedrunPhase.RUNNING }
            .stateIn(scope, SharingStarted.Eagerly, _uiState.value.phase == SpeedrunPhase.RUNNING)

    init {
        // A single long-lived collector. While NOT_STARTED it keeps the Start gate up to date;
        // while RUNNING it advances the task index.
        scope.launch {
            combine(
                    deviceRepository.deviceState,
                    connectionProxy.tunnelState,
                    settingsRepository.settingsUpdates,
                ) { device, tunnel, settings ->
                    DaemonSnapshot(device, tunnel, settings)
                }
                .distinctUntilChanged()
                .collect { snapshot -> onSnapshot(snapshot) }
        }
    }

    /** Invoked from the overlay's Start button. No-op unless idle and not known-dirty. */
    fun start() {
        val nowWall = System.currentTimeMillis()
        val nowElapsed = SystemClock.elapsedRealtime()
        val started = _uiState.updateAndGet { current ->
            if (
                current.phase != SpeedrunPhase.NOT_STARTED ||
                    (current.baselineKnown && !current.baselineClean)
            ) {
                current
            } else {
                current.copy(
                    phase = SpeedrunPhase.RUNNING,
                    startWallMillis = nowWall,
                    startElapsedMillis = nowElapsed,
                    currentIndex = 0,
                )
            }
        }
        if (started.phase == SpeedrunPhase.RUNNING && started.startWallMillis == nowWall) {
            Logger.i { "Speedrun: started" }
            clearClipboard()
            persist(started)
        }
    }

    private fun onSnapshot(snapshot: DaemonSnapshot) {
        when (_uiState.value.phase) {
            SpeedrunPhase.NOT_STARTED -> refreshBaseline(snapshot)
            SpeedrunPhase.RUNNING -> advance(snapshot)
            SpeedrunPhase.FINISHED -> Unit
        }
    }

    private fun refreshBaseline(snapshot: DaemonSnapshot) {
        // Guarded so a concurrent start() is never clobbered. Baseline flags are not persisted.
        _uiState.update { current ->
            if (current.phase != SpeedrunPhase.NOT_STARTED) current
            else
                current.copy(
                    baselineKnown = isBaselineKnown(snapshot),
                    baselineClean = isCleanBaseline(snapshot),
                )
        }
    }

    private fun advance(snapshot: DaemonSnapshot) {
        val before = _uiState.value
        if (before.phase != SpeedrunPhase.RUNNING) return

        var index = before.currentIndex
        while (index < SPEEDRUN_TASKS.size && SPEEDRUN_TASKS[index].isComplete(snapshot)) {
            Logger.i { "Speedrun: completed step ${index + 1} (${SPEEDRUN_TASKS[index].title})" }
            index++
        }
        if (index == before.currentIndex) return

        val finished = index >= SPEEDRUN_TASKS.size
        val nowWall = System.currentTimeMillis()
        val nowElapsed = SystemClock.elapsedRealtime()
        val updated = _uiState.updateAndGet { current ->
            when {
                current.phase != SpeedrunPhase.RUNNING -> current
                finished ->
                    current.copy(
                        phase = SpeedrunPhase.FINISHED,
                        finishWallMillis = nowWall,
                        finishElapsedMillis = nowElapsed,
                        currentIndex = SPEEDRUN_TASKS.size,
                    )
                else -> current.copy(currentIndex = index)
            }
        }
        if (finished) {
            Logger.i { "Speedrun: finished in ${updated.finalElapsedMillis} ms" }
        }
        persist(updated)
    }

    private fun clearClipboard() {
        // Wipe any pre-staged account number so it cannot be pasted during the login step.
        runCatching { clipboard?.clearPrimaryClip() }
    }

    private fun persist(state: SpeedrunUiState) {
        prefs
            .edit()
            .putString(KEY_PHASE, state.phase.name)
            .putLong(KEY_START_WALL, state.startWallMillis)
            .putLong(KEY_START_ELAPSED, state.startElapsedMillis)
            .putLong(KEY_FINISH_WALL, state.finishWallMillis)
            .putLong(KEY_FINISH_ELAPSED, state.finishElapsedMillis)
            .putInt(KEY_INDEX, state.currentIndex)
            .apply()
    }

    private fun loadPersistedState(): SpeedrunUiState {
        val phase =
            prefs.getString(KEY_PHASE, null)?.let {
                runCatching { SpeedrunPhase.valueOf(it) }.getOrNull()
            } ?: SpeedrunPhase.NOT_STARTED
        return SpeedrunUiState(
            phase = phase,
            startWallMillis = prefs.getLong(KEY_START_WALL, 0L),
            startElapsedMillis = prefs.getLong(KEY_START_ELAPSED, 0L),
            finishWallMillis = prefs.getLong(KEY_FINISH_WALL, 0L),
            finishElapsedMillis = prefs.getLong(KEY_FINISH_ELAPSED, 0L),
            currentIndex = prefs.getInt(KEY_INDEX, 0),
        )
    }

    private companion object {
        const val PREFS_NAME = "speedrun_competition"
        const val KEY_PHASE = "phase"
        const val KEY_START_WALL = "start_wall_millis"
        const val KEY_START_ELAPSED = "start_elapsed_millis"
        const val KEY_FINISH_WALL = "finish_wall_millis"
        const val KEY_FINISH_ELAPSED = "finish_elapsed_millis"
        const val KEY_INDEX = "current_index"
    }
}
