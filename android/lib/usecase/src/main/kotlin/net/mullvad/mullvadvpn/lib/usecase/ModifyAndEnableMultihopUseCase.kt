package net.mullvad.mullvadvpn.lib.usecase

import arrow.core.Either
import arrow.core.raise.either
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class ModifyAndEnableMultihopUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val customListsRepository: CustomListsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    suspend operator fun invoke(
        multihopMode: MultihopMode,
        change: RelayMultihopChange,
    ): Either<ModifyMultihopError, Unit> = either {
        validate(
                change = change,
                settingsRepository = settingsRepository,
                customListsRepository = customListsRepository,
            )
            .bind()
        when (change) {
                is RelayMultihopChange.Entry ->
                    wireguardConstraintsRepository.setMultihopAndEntryLocation(
                        multihopMode,
                        change.item.id,
                    )
                is RelayMultihopChange.Exit ->
                    relayListRepository.updateExitRelayLocationMultihop(
                        multihopMode,
                        change.item.id,
                    )
            }
            .mapLeft {
                Logger.e("Failed to update multihop: $it")
                ModifyMultihopError.GenericError
            }
            .bind()
    }
}
