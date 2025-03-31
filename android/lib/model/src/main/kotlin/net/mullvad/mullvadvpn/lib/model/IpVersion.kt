package net.mullvad.mullvadvpn.lib.model

enum class IpVersion {
    IPV4,
    IPV6;

    companion object {
        val constraints: List<Constraint<IpVersion>> = buildList {
            add(Constraint.Any)
            addAll(IpVersion.entries.map { Constraint.Only(it) })
        }
    }
}
