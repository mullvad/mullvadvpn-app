package net.mullvad.mullvadvpn.speedrun

import android.content.Context
import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

enum class SpeedrunPhase {
    NOT_STARTED,
    RUNNING,
    FINISHED,
}

data class SpeedrunUiState(
    val phase: SpeedrunPhase = SpeedrunPhase.NOT_STARTED,
    val startTimeMillis: Long = 0L,
    val finishTimeMillis: Long = 0L,
    val currentIndex: Int = 0,
    // Live (not persisted) flags used only to gate the Start button.
    val baselineKnown: Boolean = false,
    val baselineClean: Boolean = false,
) {
    val totalTasks: Int = SPEEDRUN_TASKS.size
    val currentTask: SpeedrunTask? = SPEEDRUN_TASKS.getOrNull(currentIndex)

    /** Final time once finished. The running time is computed live by the overlay instead. */
    val finalElapsedMillis: Long = (finishTimeMillis - startTimeMillis).coerceAtLeast(0L)
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
) {
    private val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _uiState = MutableStateFlow(loadPersistedState())
    val uiState: StateFlow<SpeedrunUiState> = _uiState.asStateFlow()

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

    /** Invoked from the overlay's Start button. No-op unless idle and the baseline is clean. */
    fun start() {
        val now = System.currentTimeMillis()
        val started = _uiState.updateAndGet { current ->
            if (
                current.phase != SpeedrunPhase.NOT_STARTED ||
                    (current.baselineKnown && !current.baselineClean)
            ) {
                current
            } else {
                current.copy(phase = SpeedrunPhase.RUNNING, startTimeMillis = now, currentIndex = 0)
            }
        }
        if (started.phase == SpeedrunPhase.RUNNING && started.startTimeMillis == now) {
            Logger.i { "Speedrun: started" }
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
        val now = System.currentTimeMillis()
        val updated = _uiState.updateAndGet { current ->
            when {
                current.phase != SpeedrunPhase.RUNNING -> current
                finished ->
                    current.copy(
                        phase = SpeedrunPhase.FINISHED,
                        finishTimeMillis = now,
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

    private fun persist(state: SpeedrunUiState) {
        prefs
            .edit()
            .putString(KEY_PHASE, state.phase.name)
            .putLong(KEY_START, state.startTimeMillis)
            .putLong(KEY_FINISH, state.finishTimeMillis)
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
            startTimeMillis = prefs.getLong(KEY_START, 0L),
            finishTimeMillis = prefs.getLong(KEY_FINISH, 0L),
            currentIndex = prefs.getInt(KEY_INDEX, 0),
        )
    }

    private companion object {
        const val PREFS_NAME = "speedrun_competition"
        const val KEY_PHASE = "phase"
        const val KEY_START = "start_time_millis"
        const val KEY_FINISH = "finish_time_millis"
        const val KEY_INDEX = "current_index"
    }
}
