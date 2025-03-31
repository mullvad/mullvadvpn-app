package net.mullvad.mullvadvpn.viewmodel

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.ServerIpOverridesDestination
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
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository

class ServerIpOverridesViewModel(
    private val relayOverridesRepository: RelayOverridesRepository,
    private val contentResolver: ContentResolver,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = ServerIpOverridesDestination.argsFrom(savedStateHandle)

    private val _uiSideEffect = Channel<ServerIpOverridesUiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    val uiState: StateFlow<ServerIpOverridesUiState> =
        relayOverridesRepository.relayOverrides
            .filterNotNull()
            .map {
                ServerIpOverridesUiState.Loaded(
                    overridesActive = it.isNotEmpty(),
                    isModal = navArgs.isModal,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ServerIpOverridesUiState.Loading(navArgs.isModal),
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
                    { _uiSideEffect.send(ServerIpOverridesUiSideEffect.ImportResult(null)) },
                )
        }
    }
}

sealed interface ServerIpOverridesUiSideEffect {
    data class ImportResult(val error: SettingsPatchError?) : ServerIpOverridesUiSideEffect
}

sealed interface ServerIpOverridesUiState {
    val overridesActive: Boolean?
        get() = (this as? Loaded)?.overridesActive

    val isModal: Boolean

    data class Loading(override val isModal: Boolean = false) : ServerIpOverridesUiState

    data class Loaded(
        override val overridesActive: Boolean,
        override val isModal: Boolean = false,
    ) : ServerIpOverridesUiState
}
