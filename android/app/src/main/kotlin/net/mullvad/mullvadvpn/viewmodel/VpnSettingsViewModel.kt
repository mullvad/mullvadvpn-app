package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.None
import arrow.core.Option
import arrow.core.Some
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.VpnSettingsDestination
import java.net.Inet6Address
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.CustomDnsItem
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.util.BackstackObserver
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.contentBlockersSettings
import net.mullvad.mullvadvpn.util.customDnsAddresses
import net.mullvad.mullvadvpn.util.deviceIpVersion
import net.mullvad.mullvadvpn.util.isCustomDnsEnabled
import net.mullvad.mullvadvpn.util.onFirst
import net.mullvad.mullvadvpn.util.quantumResistant
import net.mullvad.mullvadvpn.util.selectedObfuscationMode
import net.mullvad.mullvadvpn.util.toLc

sealed interface VpnSettingsSideEffect {
    sealed interface ShowToast : VpnSettingsSideEffect {
        data object ApplySettingsWarning : ShowToast

        data object GenericError : ShowToast
    }

    data object NavigateToDnsDialog : VpnSettingsSideEffect
}

@Suppress("TooManyFunctions")
class VpnSettingsViewModel(
    private val settingsRepository: SettingsRepository,
    private val systemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase,
    private val autoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    savedStateHandle: SavedStateHandle,
    backstackObserver: BackstackObserver,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = VpnSettingsDestination.argsFrom(savedStateHandle)
    private val _mutableIsContentBlockersExpanded = MutableStateFlow<Option<Boolean>>(None)

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState =
        combine(
                settingsRepository.settingsUpdates.filterNotNull().onFirst {
                    // Initialize content blockers expand state
                    _mutableIsContentBlockersExpanded.value =
                        Some(it.contentBlockersSettings().isAnyBlockerEnabled())
                },
                autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot,
                _mutableIsContentBlockersExpanded.filterIsInstance<Some<Boolean>>().map {
                    it.value
                },
                backstackObserver.previousDestinationFlow.map { it is ConnectDestination },
            ) {
                settings,
                autoStartAndConnectOnBoot,
                isContentBlockersExpanded,
                isScrollToFeatureEnabled ->
                VpnSettingsUiState.from(
                        mtu = settings.tunnelOptions.mtu,
                        isLocalNetworkSharingEnabled = settings.allowLan,
                        isCustomDnsEnabled = settings.isCustomDnsEnabled(),
                        customDnsItems = settings.customDnsAddresses().asStringAddressList(),
                        contentBlockersOptions = settings.contentBlockersSettings(),
                        obfuscationMode = settings.selectedObfuscationMode(),
                        quantumResistant = settings.quantumResistant(),
                        systemVpnSettingsAvailable = systemVpnSettingsUseCase(),
                        autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
                        deviceIpVersion = settings.deviceIpVersion(),
                        isIpv6Enabled = settings.tunnelOptions.enableIpv6,
                        isContentBlockersExpanded = isContentBlockersExpanded,
                        isModal = navArgs.isModal,
                        isScrollToFeatureEnabled = isScrollToFeatureEnabled,
                    )
                    .toLc<Boolean, VpnSettingsUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(navArgs.isModal),
            )

    fun onToggleLocalNetworkSharing(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            settingsRepository.setLocalNetworkSharing(isEnabled).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onToggleCustomDns(enable: Boolean) =
        viewModelScope.launch {
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
                viewModelScope.launch {
                    _uiSideEffect.send(VpnSettingsSideEffect.NavigateToDnsDialog)
                }
            }
        }

    fun onToggleContentBlockersExpand() =
        _mutableIsContentBlockersExpanded.update { it.map { expand -> !expand } }

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

    fun onSelectQuantumResistanceSetting(enable: Boolean) {
        viewModelScope.launch(dispatcher) {
            settingsRepository
                .setWireguardQuantumResistant(
                    if (enable) {
                        QuantumResistantState.On
                    } else {
                        QuantumResistantState.Off
                    }
                )
                .onLeft { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
        }
    }

    fun onToggleAutoStartAndConnectOnBoot(autoStartAndConnect: Boolean) =
        viewModelScope.launch(dispatcher) {
            autoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(autoStartAndConnect)
        }

    fun onDeviceIpVersionSelected(ipVersion: Constraint<IpVersion>) =
        viewModelScope.launch(dispatcher) {
            wireguardConstraintsRepository.setDeviceIpVersion(ipVersion).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }

    fun setIpv6Enabled(enable: Boolean) =
        viewModelScope.launch(dispatcher) {
            settingsRepository.setIpv6Enabled(enable).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }

    private fun updateContentBlockersAndNotify(update: (DefaultDnsOptions) -> DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            settingsRepository
                .updateContentBlockers(update)
                .fold(
                    {
                        Logger.e("Failed to update content blockers")
                        _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
                    },
                    { showApplySettingChangesWarningToast() },
                )
        }

    private fun List<InetAddress>.asStringAddressList(): List<CustomDnsItem> = map {
        CustomDnsItem(
            address = it.hostAddress ?: EMPTY_STRING,
            isLocal = it.isLocalAddress(),
            isIpv6 = it is Inet6Address,
        )
    }

    private fun InetAddress.isLocalAddress(): Boolean = isLinkLocalAddress || isSiteLocalAddress

    fun showApplySettingChangesWarningToast() =
        viewModelScope.launch {
            _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.ApplySettingsWarning)
        }

    fun showGenericErrorToast() =
        viewModelScope.launch { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }

    companion object {
        private const val EMPTY_STRING = ""
    }
}
