package net.mullvad.mullvadvpn.lib.model

data class IncompatibleConstraints(
    val inactive: Boolean,
    val location: Boolean,
    val providers: Boolean,
    val ownership: Boolean,
    val ipVersion: Boolean,
    val daita: Boolean,
    val obfuscation: Boolean,
    val port: Boolean,
    val conflictWithOtherHop: Boolean,
) {
    val onlyInactive =
        (conflictWithOtherHop or inactive) &&
            !location &&
            !providers &&
            !ownership &&
            !ipVersion &&
            !daita &&
            !obfuscation &&
            !port
}
