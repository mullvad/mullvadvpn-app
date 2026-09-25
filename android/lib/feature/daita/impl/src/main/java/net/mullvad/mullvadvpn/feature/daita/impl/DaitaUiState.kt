package net.mullvad.mullvadvpn.feature.daita.impl

data class DaitaUiState(
    val daitaMode: DaitaMode,
    val isModal: Boolean = false,
)

sealed interface DaitaMode {
    data object Always : DaitaMode
    data object UnMetered : DaitaMode
    data object Off : DaitaMode
}
