package net.mullvad.mullvadvpn.feature.appinfo.impl

import android.content.res.Resources
import androidx.core.net.toUri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.AppVersionInfoRepository
import net.mullvad.mullvadvpn.lib.ui.resource.R

class AppInfoViewModel(
    appVersionInfoRepository: AppVersionInfoRepository,
    private val resources: Resources,
    private val isPlayBuild: Boolean,
    private val isFdroidBuild: Boolean,
    private val packageName: String,
) : ViewModel() {

    private val _uiSideEffect = Channel<AppInfoSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<Unit, AppInfoUiState>> =
        appVersionInfoRepository.versionInfo
            .map { versionInfo -> Lc.Content(AppInfoUiState(versionInfo, isPlayBuild)) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun openAppListing() =
        viewModelScope.launch {
            val sideEffect =
                if (isPlayBuild || isFdroidBuild) {
                    AppInfoSideEffect.OpenUri(
                        uri = resources.getString(R.string.market_uri, packageName).toUri(),
                        errorMessage = resources.getString(R.string.uri_market_app_not_found),
                    )
                } else {
                    AppInfoSideEffect.OpenUri(
                        uri = resources.getString(R.string.download_url).toUri(),
                        errorMessage = resources.getString(R.string.uri_browser_app_not_found),
                    )
                }
            _uiSideEffect.send(sideEffect)
        }
}
