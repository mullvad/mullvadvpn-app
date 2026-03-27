package net.mullvad.mullvadvpn.feature.settings.impl

import androidx.compose.remote.core.CoreDocument
import androidx.compose.remote.creation.RemoteComposeWriter
import androidx.compose.remote.creation.compose.capture.RemoteComposeCapture
import androidx.compose.remote.creation.compose.widgets.RemoteComposeWidget
import androidx.compose.remote.player.core.RemoteDocument
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.settings.impl.server.Server
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT

data class FaqRemoteState(val document: CoreDocument)

class FaqRemoteViewModel(private val server: Server) : ViewModel() {

    private val document = MutableStateFlow<CoreDocument?>(null)

    val uiState =
        document
            .filterNotNull()
            .map { Lc.Content(FaqRemoteState(document = it)) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    init {
        viewModelScope.launch(Dispatchers.IO) {
            val remoteDocument = server.main()
            //val bytes = remoteDocument.bytes

            //val playerDocument = CoreDocument()
            Logger.d { "remoteDocument:$remoteDocument" }

            //playerDocument.
            //document.value = playerDocument
            //RemoteComposeCapture
            document.value = RemoteDocument(remoteDocument.bytes).document
        }
    }
}
