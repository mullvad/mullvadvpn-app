package net.mullvad.mullvadvpn.feature.appinfo.impl

import androidx.core.net.toUri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.applisting.api.ResolveAppListingUseCase
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Scenario
import net.mullvad.mullvadvpn.lib.repository.AppVersionInfoRepository
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository
import net.mullvad.mullvadvpn.lib.usecase.MultihopGuideMigrationHintUseCase

class AppInfoViewModel(
    appVersionInfoRepository: AppVersionInfoRepository,
    private val isPlayBuild: Boolean,
    private val resolveAppListing: ResolveAppListingUseCase,
    multihopMigrationRepository: MultihopMigrationRepository,
    multihopGuideMigrationHintUseCase: MultihopGuideMigrationHintUseCase,
) : ViewModel() {

    private val _uiSideEffect = Channel<AppInfoSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<Unit, AppInfoUiState>> =
        combine(
                appVersionInfoRepository.versionInfo,
                multihopMigrationRepository.multihopMigrationState,
                multihopGuideMigrationHintUseCase(),
            ) { versionInfo, splitFilterMigration, showMigrationGuideNotificationDot ->
                Lc.Content(
                    AppInfoUiState(
                        version = versionInfo,
                        splitFilterMigration =
                            splitFilterMigration?.takeUnless {
                                it.scenario == null || it.scenario == Scenario.ONE_A
                            },
                        isPlayBuild = isPlayBuild,
                        showMigrationGuideNotificationDot = showMigrationGuideNotificationDot,
                    )
                )
            }
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
