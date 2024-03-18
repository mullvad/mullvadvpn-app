package net.mullvad.mullvadvpn.viewmodel

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.nio.charset.Charset
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
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
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState

class ServerIpOverridesViewModel(
    private val contentResolver: ContentResolver,
    private val serviceConnectionManager: ServiceConnectionManager,
    relayOverridesRepository: RelayOverridesRepository,
    private val settingsRepository: SettingsRepository
) : ViewModel() {

    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    val uiState: StateFlow<ServerIpOverridesViewState> =
        relayOverridesRepository.relayOverrides
            .map { ServerIpOverridesViewState(overridesActive = it?.isNotEmpty() ?: false) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ServerIpOverridesViewState(overridesActive = false)
            )

    fun importFile(uri: Uri) =
        viewModelScope.launch {
            // Read json from file
            val json =
                contentResolver.openInputStream(uri)!!.reader(Charset.defaultCharset()).readText()
            applySettingsPatch(json)
        }

    fun importText(config: String) = viewModelScope.launch { applySettingsPatch(config) }

    private suspend fun applySettingsPatch(json: String) {
        // Wait for daemon to come online since we might be disconnected (due to File picker being
        // open
        // and we disconnect from daemon in paused state)
        val connResult =
            withTimeoutOrNull(5.seconds) {
                serviceConnectionManager.connectionState
                    .filterIsInstance(ServiceConnectionState.ConnectedReady::class)
                    .first()
            }
        if (connResult != null) {
            // Apply patch
            val result = settingsRepository.applySettingsPatch(json)
            _uiSideEffect.send(UiSideEffect.ImportResult(result.error))
        } else {
            // Service never came online, at this point we should already display daemon overlay
        }
    }

    sealed interface UiSideEffect {
        data class ImportResult(val error: SettingsPatchError?) : UiSideEffect

        data object OverridesCleared : UiSideEffect
    }
}

data class ServerIpOverridesViewState(val overridesActive: Boolean)
