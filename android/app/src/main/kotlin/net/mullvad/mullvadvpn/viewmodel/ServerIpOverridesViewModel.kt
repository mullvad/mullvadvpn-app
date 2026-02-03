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
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.lib.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class ServerIpOverridesViewModel(
    private val relayOverridesRepository: RelayOverridesRepository,
    private val contentResolver: ContentResolver,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = ServerIpOverridesDestination.argsFrom(savedStateHandle)

    private val _uiSideEffect = Channel<ServerIpOverridesUiSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    val uiState: StateFlow<Lc<Boolean, ServerIpOverridesUiState>> =
        relayOverridesRepository.relayOverrides
            .filterNotNull()
            .map {
                ServerIpOverridesUiState(
                        overridesActive = it.isNotEmpty(),
                        isModal = navArgs.isModal,
                    )
                    .toLc<Boolean, ServerIpOverridesUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(navArgs.isModal),
            )

    fun importFile(uri: Uri) =
        viewModelScope.launch {
            // Read json from file
            val inputStream = contentResolver.openInputStream(uri)!!
            val json = InputStreamReader(inputStream, Charsets.UTF_8).readText()

            applySettingsPatch(json)
        }

    fun importText(json: String) = viewModelScope.launch { applySettingsPatch(json) }

    private fun applySettingsPatch(json: String) {
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

data class ServerIpOverridesUiState(val overridesActive: Boolean, val isModal: Boolean = false)
