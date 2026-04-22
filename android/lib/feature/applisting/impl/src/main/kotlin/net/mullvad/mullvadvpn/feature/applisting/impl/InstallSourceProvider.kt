package net.mullvad.mullvadvpn.feature.applisting.impl

fun interface InstallSourceProvider {
    fun isInstalledFromStore(): Boolean
}
