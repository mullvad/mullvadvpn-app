package net.mullvad.mullvadvpn.model

data class RelayConstraintsUpdate(
    val location: Constraint<LocationConstraint>?,
    val providers: Constraint<Providers>?,
    val ownership: Constraint<Ownership>?,
    val wireguardConstraints: WireguardConstraints?
)
