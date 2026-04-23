package net.mullvad.mullvadvpn.feature.dns.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import java.net.Inet6Address
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.contentBlockersSettings
import net.mullvad.mullvadvpn.lib.common.util.customDnsAddresses
import net.mullvad.mullvadvpn.lib.common.util.isCustomDnsEnabled
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.ui.component.EMPTY_STRING

data class DnsSettingsUiState(
    val isModal: Boolean,
    val contentBlockersExpanded: Boolean,
    val contentBlockersEnabled: Boolean,
    val defaultDnsOptions: DefaultDnsOptions,
    val customDnsEnabled: Boolean,
    val customDnsEntries: List<CustomDnsEntry>,
    val showUnreachableLocalDnsWarning: Boolean,
    val showUnreachableIpv6DnsWarning: Boolean,
)

data class CustomDnsEntry(val address: String, val isLocal: Boolean, val isIpv6: Boolean)

sealed interface DnsSettingsSideEffect {
    data object NavigateToDnsDialog : DnsSettingsSideEffect

    sealed interface ShowToast : DnsSettingsSideEffect {
        data object ApplySettingWarning : ShowToast

        data object GenericError : ShowToast
    }
}

@Suppress("TooManyFunctions")
class DnsSettingsViewModel(
    private val isModal: Boolean,
    private val settingsRepository: SettingsRepository,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val _contentBlockersExpanded = MutableStateFlow(true)

    private val _uiSideEffect = Channel<DnsSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<Unit, DnsSettingsUiState>> =
        combine(settingsRepository.settingsUpdates.filterNotNull(), _contentBlockersExpanded) {
                settings,
                contentBlockersExpanded ->
                Lc.Content(
                    DnsSettingsUiState(
                        isModal = isModal,
                        contentBlockersExpanded = contentBlockersExpanded,
                        contentBlockersEnabled = !settings.isCustomDnsEnabled(),
                        defaultDnsOptions = settings.contentBlockersSettings(),
                        customDnsEnabled = settings.isCustomDnsEnabled(),
                        customDnsEntries = settings.customDnsAddresses().asStringAddressList(),
                        showUnreachableLocalDnsWarning = !settings.allowLan,
                        showUnreachableIpv6DnsWarning = !settings.tunnelOptions.enableIpv6,
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun onToggleAllBlockers(isEnabled: Boolean) = updateContentBlockersAndNotify {
        DefaultDnsOptions(
            blockAds = isEnabled,
            blockTrackers = isEnabled,
            blockMalware = isEnabled,
            blockAdultContent = isEnabled,
            blockGambling = isEnabled,
            blockSocialMedia = isEnabled,
        )
    }

    fun onToggleBlockAds(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockAds = isEnabled)
    }

    fun onToggleBlockTrackers(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockTrackers = isEnabled)
    }

    fun onToggleBlockMalware(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockMalware = isEnabled)
    }

    fun onToggleBlockAdultContent(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockAdultContent = isEnabled)
    }

    fun onToggleBlockGambling(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockGambling = isEnabled)
    }

    fun onToggleBlockSocialMedia(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockSocialMedia = isEnabled)
    }

    fun onToggleCustomDns(enable: Boolean) = viewModelScope.launch {
        val settings = settingsRepository.settingsUpdates.value
        if (settings == null) {
            showGenericErrorToast()
            return@launch
        }

        val hasDnsEntries = settings.customDnsAddresses().isNotEmpty()

        if (hasDnsEntries) {
            settingsRepository
                .setDnsState(if (enable) DnsState.Custom else DnsState.Default)
                .fold({ showGenericErrorToast() }, { showApplySettingChangesWarningToast() })
        } else {
            // If they enable custom DNS and has no current entries we show the dialog
            // to add one.
            viewModelScope.launch { _uiSideEffect.send(DnsSettingsSideEffect.NavigateToDnsDialog) }
        }
    }

    fun onToggleContentBlockersExpanded() {
        _contentBlockersExpanded.update {
            !it
        }
    }

    fun showApplySettingChangesWarningToast() = viewModelScope.launch {
        _uiSideEffect.send(DnsSettingsSideEffect.ShowToast.ApplySettingWarning)
    }

    fun showGenericErrorToast() = viewModelScope.launch {
        _uiSideEffect.send(DnsSettingsSideEffect.ShowToast.GenericError)
    }

    private fun updateContentBlockersAndNotify(update: (DefaultDnsOptions) -> DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            settingsRepository
                .updateContentBlockers(update)
                .fold(
                    {
                        Logger.e("Failed to update content blockers")
                        _uiSideEffect.send(DnsSettingsSideEffect.ShowToast.GenericError)
                    },
                    { showApplySettingChangesWarningToast() },
                )
        }

    private fun List<InetAddress>.asStringAddressList(): List<CustomDnsEntry> = map {
        CustomDnsEntry(
            address = it.hostAddress ?: EMPTY_STRING,
            isLocal = it.isLocalAddress(),
            isIpv6 = it is Inet6Address,
        )
    }

    private fun InetAddress.isLocalAddress(): Boolean = isLinkLocalAddress || isSiteLocalAddress
}
