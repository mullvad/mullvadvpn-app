package net.mullvad.mullvadvpn.usecase

import arrow.core.raise.either
import arrow.core.raise.ensure
import net.mullvad.mullvadvpn.lib.model.SetDnsOptionsError
import net.mullvad.mullvadvpn.repository.SettingsRepository

class DeleteCustomDnsUseCase(private val settingsRepository: SettingsRepository) {
    suspend operator fun invoke(index: Int) =
        either<SetDnsOptionsError, Int> {
            val sizePriorToDeletion =
                settingsRepository.settingsUpdates.value
                    ?.tunnelOptions
                    ?.dnsOptions
                    ?.customOptions
                    ?.addresses
                    ?.size ?: 0
            ensure(sizePriorToDeletion > 0) {
                SetDnsOptionsError.Unknown(IllegalStateException("No custom DNS entries"))
            }
            settingsRepository.deleteCustomDns(index).bind()
            sizePriorToDeletion - 1
        }
}
