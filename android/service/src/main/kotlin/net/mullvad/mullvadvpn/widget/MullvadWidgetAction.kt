package net.mullvad.mullvadvpn.widget

import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.DnsState

class MullvadWidgetAction(private val managementService: ManagementService) {

    suspend fun setBlockAds(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(defaultOptions = dnsOptions.defaultOptions.copy(blockAds = enabled))
        )
    }

    suspend fun setBlockTrackers(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(
                defaultOptions = dnsOptions.defaultOptions.copy(blockTrackers = enabled)
            )
        )
    }

    suspend fun setBlockMalware(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(defaultOptions = dnsOptions.defaultOptions.copy(blockMalware = enabled))
        )
    }

    suspend fun setBlockAdultContent(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(
                defaultOptions = dnsOptions.defaultOptions.copy(blockAdultContent = enabled)
            )
        )
    }

    suspend fun setBlockGambling(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(
                defaultOptions = dnsOptions.defaultOptions.copy(blockGambling = enabled)
            )
        )
    }

    suspend fun setBlockSocialMedia(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(
                defaultOptions = dnsOptions.defaultOptions.copy(blockSocialMedia = enabled)
            )
        )
    }

    suspend fun setLan(enabled: Boolean) {
        managementService.setAllowLan(enabled)
    }

    suspend fun setCustomDns(enabled: Boolean) {
        val settings = managementService.settings.first()
        val dnsOptions = settings.tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            dnsOptions.copy(
                state =
                    if (enabled) {
                        DnsState.Custom
                    } else {
                        DnsState.Default
                    }
            )
        )
    }

    suspend fun setDaita(enabled: Boolean) {
        managementService.setDaitaEnabled(enabled)
    }

    suspend fun setSplitTunneling(enabled: Boolean) {
        managementService.setSplitTunnelingState(enabled)
    }
}
