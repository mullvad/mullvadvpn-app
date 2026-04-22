package net.mullvad.mullvadvpn.feature.appinfo.impl

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
import net.mullvad.mullvadvpn.feature.applisting.api.ResolveAppListingUseCase
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.AppVersionInfoRepository

class AppInfoViewModel(
    appVersionInfoRepository: AppVersionInfoRepository,
    private val isPlayBuild: Boolean,
    private val resolveAppListing: ResolveAppListingUseCase,
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

    fun openAppListing() = viewModelScope.launch {
        val target = resolveAppListing()
        val sideEffect =
            AppInfoSideEffect.OpenUri(
                uri = target.listingUri.toUri(),
                errorMessage = target.errorMessage,
            )
        _uiSideEffect.send(sideEffect)
    }
}
