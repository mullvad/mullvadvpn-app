package net.mullvad.mullvadvpn.app.tile

sealed interface TileState {
    val subtitle: String

    data class Active(override val subtitle: String) : TileState

    data class Inactive(override val subtitle: String) : TileState

    data class Unavailable(override val subtitle: String) : TileState
}
