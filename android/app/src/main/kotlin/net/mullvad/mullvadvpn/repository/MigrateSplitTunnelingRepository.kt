package net.mullvad.mullvadvpn.repository

import android.annotation.SuppressLint
import android.content.Context
import java.io.File
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AppId

class MigrateSplitTunnelingRepository(
    private val context: Context,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val _migrationState =
        MutableStateFlow<MigrateSplitTunnelingState>(MigrateSplitTunnelingState.Idle)
    val migrationState: StateFlow<MigrateSplitTunnelingState> = _migrationState

    private val scope: CoroutineScope = CoroutineScope(dispatcher)

    fun migrateSplitTunneling() {
        scope.launch {
            _migrationState.emit(MigrateSplitTunnelingState.Migrating)
            // Get from shared preferences, if not found return
            val (enabled, apps) =
                getOldSettings(context)
                    ?: run {
                        _migrationState.emit(MigrateSplitTunnelingState.Idle)
                        return@launch
                    }

            // Set new settings, if failed return
            if (splitTunnelingRepository.enableSplitTunneling(enabled).isLeft()) {
                _migrationState.emit(MigrateSplitTunnelingState.Failure)
                return@launch
            }
            if (splitTunnelingRepository.excludeApps(apps.map { AppId(it) }).isLeft()) {
                _migrationState.emit(MigrateSplitTunnelingState.Failure)
                return@launch
            }

            // Remove old settings
            removeOldSettings(context)

            _migrationState.emit(MigrateSplitTunnelingState.Success)
        }
    }

    fun clearOldSettings() {
        removeOldSettings(context)
    }

    private fun getOldSettings(context: Context): Pair<Boolean, Set<String>>? {
        // Get from shared preferences and appListFile
        val appListFile = File(context.filesDir, "split-tunnelling.txt")
        val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

        return when {
            !appListFile.exists() -> return null
            !preferences.contains(KEY_ENABLED) -> return null
            else -> preferences.getBoolean(KEY_ENABLED, false) to appListFile.readLines().toSet()
        }
    }

    @SuppressLint("ApplySharedPref")
    private fun removeOldSettings(context: Context) {
        // Remove from shared preferences and app list file
        val appListFile = File(context.filesDir, "split-tunnelling.txt")
        val preferences = context.getSharedPreferences(SHARED_PREFERENCES, Context.MODE_PRIVATE)

        appListFile.delete()
        preferences.edit().remove(KEY_ENABLED).apply()
    }
}

private const val SHARED_PREFERENCES = "split_tunnelling"
private const val KEY_ENABLED = "enabled"

sealed interface MigrateSplitTunnelingState {
    data object Idle : MigrateSplitTunnelingState

    data object Migrating : MigrateSplitTunnelingState

    data object Success : MigrateSplitTunnelingState

    data object Failure : MigrateSplitTunnelingState
}
