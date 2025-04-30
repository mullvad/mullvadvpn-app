package net.mullvad.mullvadvpn.lib.model

data class SuggestedUpgrade(val version: String, val changelog: String, val verifiedInstallerPath: String?)
