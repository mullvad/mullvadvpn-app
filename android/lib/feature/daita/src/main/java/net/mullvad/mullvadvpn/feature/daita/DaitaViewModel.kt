package net.mullvad.mullvadvpn.feature.daita

class DaitaViewModel(
    private val settingsRepository: SettingsRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val navArgs = DaitaDestination.argsFrom(savedStateHandle)

    val uiState =
        settingsRepository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                DaitaUiState(
                        daitaEnabled = settings.isDaitaEnabled(),
                        directOnly = settings.isDaitaDirectOnly(),
                        navArgs.isModal,
                    )
                    .toLc<Boolean, DaitaUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(navArgs.isModal),
            )

    fun setDaita(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaEnabled(enable) }
    }

    fun setDirectOnly(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaDirectOnly(enable) }
    }
}
