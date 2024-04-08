package net.mullvad.mullvadvpn.viewmodel

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.io.InputStreamReader
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.SettingsPatchError
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository

class ServerIpOverridesViewModel(
    val relayOverridesRepository: RelayOverridesRepository,
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
        // Since we are currently using waitForReady this will just wait to apply until gRPC is
        // ready
        viewModelScope.launch {
            relayOverridesRepository
                .applySettingsPatch(json)
                .fold(
                    { error ->
                        _uiSideEffect.send(ServerIpOverridesUiSideEffect.ImportResult(error))
                    },
                    { _uiSideEffect.send(ServerIpOverridesUiSideEffect.ImportResult(null)) }
                )
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
