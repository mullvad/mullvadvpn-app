package net.mullvad.mullvadvpn.model

data class RelayConstraintsUpdate(
    val location: Constraint<LocationConstraint>?,
    val ownership: Constraint<Ownership>?,
    val wireguardConstraints: WireguardConstraints?
)
