package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Settings

fun Settings.quantumResistant() = tunnelOptions.quantumResistant

fun Settings.isCustomDnsEnabled() = tunnelOptions.dnsOptions.state == DnsState.Custom

fun Settings.wireguardConstraints() = relaySettings.relayConstraints.wireguardConstraints

fun Settings.customDnsAddresses() = tunnelOptions.dnsOptions.customOptions.addresses

fun Settings.contentBlockersSettings() = tunnelOptions.dnsOptions.defaultOptions

fun Settings.selectedObfuscationMode() = obfuscationSettings.selectedObfuscationMode

fun Settings.wireguardPort() = obfuscationSettings.port

fun Settings.deviceIpVersion() = relaySettings.relayConstraints.wireguardConstraints.ipVersion

fun Settings.isDaitaAndDirectOnly() = isDaitaEnabled() && isDaitaDirectOnly()

fun Settings.isDaitaAndNotDirectOnly() = isDaitaEnabled() && !isDaitaDirectOnly()

fun Settings.isQuicEnabled() = obfuscationSettings.selectedObfuscationMode == ObfuscationMode.Quic

fun Settings.isLwoEnabled() = obfuscationSettings.selectedObfuscationMode == ObfuscationMode.Lwo

fun Settings.ipVersionConstraint() = relaySettings.relayConstraints.wireguardConstraints.ipVersion

fun Settings.isDaitaEnabled() = daitaSettings().enabled

fun Settings.isDaitaDirectOnly() = daitaSettings().directOnly

fun Settings.shadowSocksPort() = obfuscationSettings.shadowsocks.port

fun Settings.isMultihopEnabled() =
    relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled

fun Settings.location() = relaySettings.relayConstraints.location

private fun Settings.daitaSettings() = tunnelOptions.daitaSettings
