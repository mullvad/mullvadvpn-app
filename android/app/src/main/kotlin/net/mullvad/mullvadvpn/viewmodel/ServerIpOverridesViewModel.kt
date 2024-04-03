package net.mullvad.mullvadvpn.viewmodel

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.io.InputStreamReader
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.model.SettingsPatchError
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager

class ServerIpOverridesViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    relayOverridesRepository: RelayOverridesRepository,
    private val settingsRepository: SettingsRepository,
    private val contentResolver: ContentResolver,
) : ViewModel() {

    private val _uiSideEffect = Channel<ServerIpOverridesUiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    val uiState: StateFlow<ServerIpOverridesViewState> =
        relayOverridesRepository.relayOverrides
            .filterNotNull()
            .map { ServerIpOverridesViewState.Loaded(overridesActive = it.isNotEmpty()) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ServerIpOverridesViewState.Loading
            )

    fun importFile(uri: Uri) =
        viewModelScope.launch {
            // Read json from file
            val inputStream = contentResolver.openInputStream(uri)!!
            val json = InputStreamReader(inputStream, Charsets.UTF_8).readText()

            applySettingsPatch(json)
        }

    fun importText(json: String) = viewModelScope.launch { applySettingsPatch(json) }

    private suspend fun applySettingsPatch(json: String) {
        // Wait for daemon to come online since we might be disconnected (due to File picker being
        // open
        // and we disconnect from daemon in paused state)
        val connResult =
            withTimeoutOrNull(5.seconds) {
                TODO("Call management service")
                //                serviceConnectionManager.connectionState
                //
                // .filterIsInstance(ServiceConnectionState.ConnectedReady::class)
                //                    .first()
            }
        if (connResult != null) {
            // Apply patch
            //            val result = settingsRepository.applySettingsPatch(json)
            TODO("Ensure patch was applied")
            //
            // _uiSideEffect.send(ServerIpOverridesUiSideEffect.ImportResult(result.error))
        } else {
            // Service never came online, at this point we should already display daemon overlay
        }
    }
}

sealed interface ServerIpOverridesUiSideEffect {
    data class ImportResult(val error: SettingsPatchError?) : ServerIpOverridesUiSideEffect
}

sealed interface ServerIpOverridesViewState {
    val overridesActive: Boolean?
        get() = (this as? Loaded)?.overridesActive

    data object Loading : ServerIpOverridesViewState

    data class Loaded(override val overridesActive: Boolean) : ServerIpOverridesViewState
}
