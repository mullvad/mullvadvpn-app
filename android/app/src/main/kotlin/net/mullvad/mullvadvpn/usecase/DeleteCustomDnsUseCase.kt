package net.mullvad.mullvadvpn.usecase

import arrow.core.raise.either
import net.mullvad.mullvadvpn.lib.model.SetDnsOptionsError
import net.mullvad.mullvadvpn.repository.SettingsRepository

class DeleteCustomDnsUseCase(private val settingsRepository: SettingsRepository) {
    suspend operator fun invoke(index: Int) =
        either<SetDnsOptionsError, Int> {
            val size =
                settingsRepository.settingsUpdates.value
                    ?.tunnelOptions
                    ?.dnsOptions
                    ?.customOptions
                    ?.addresses
                    ?.size ?: 0
            settingsRepository.deleteCustomDns(index).bind()
            size - 1
        }
}
