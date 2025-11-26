package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.core.raise.ensureNotNull
import arrow.core.right
import co.touchlab.kermit.Logger
import kotlin.collections.first
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled

class ModifyAndEnableMultihopUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val customListsRepository: CustomListsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    suspend operator fun invoke(
        enableMultihop: Boolean,
        change: MultihopChange,
    ): Either<ModifyMultihopError, Unit> = either {
        ensure(change.item.active) { ModifyMultihopError.RelayItemInactive(change.item) }
        val changeId: RelayItemId =
            change.item.id.convertCustomListWithOnlyHostNameToHostName().bind()
        val settings = settingsRepository.settingsUpdates.value
        ensureNotNull(settings) { ModifyMultihopError.GenericError }
        val other =
            when (change) {
                    is MultihopChange.Entry -> settings.exit()
                    is MultihopChange.Exit -> settings.entry()
                }
                ?.convertCustomListWithOnlyHostNameToHostName()
                ?.bind()
        // If DAITA is enabled and direct only is disabled, allow same relay for entry and
        // exit.
        if (!settings.isDaitaEnabled() || settings.isDaitaDirectOnly()) {
            ensure(!changeId.isSameHost(other)) { ModifyMultihopError.EntrySameAsExit(change.item) }
        }
        when (change) {
                is MultihopChange.Entry ->
                    wireguardConstraintsRepository.setMultihopAndEntryLocation(
                        enableMultihop,
                        change.item.id,
                    )
                is MultihopChange.Exit ->
                    relayListRepository.updateExitRelayLocationMultihop(
                        enableMultihop,
                        change.item.id,
                    )
            }
            .mapLeft {
                Logger.e("Failed to update multihop: $it")
                ModifyMultihopError.GenericError
            }
            .bind()
    }

    private fun Settings.exit(): RelayItemId? = relaySettings.relayConstraints.location.getOrNull()

    private fun Settings.entry(): RelayItemId? =
        relaySettings.relayConstraints.wireguardConstraints.entryLocation.getOrNull()

    private fun RelayItemId.convertCustomListWithOnlyHostNameToHostName():
        Either<ModifyMultihopError.GenericError, RelayItemId> =
        when (this) {
            is CustomListId ->
                customListsRepository
                    .getCustomListById(this)
                    .mapLeft {
                        Logger.e("Failed to get custom list by id: $it")
                        ModifyMultihopError.GenericError
                    }
                    .map {
                        if (it.locations.size == 1) {
                            it.locations.first() as? GeoLocationId.Hostname ?: this
                        } else {
                            this
                        }
                    }
            else -> this.right()
        }

    private fun RelayItemId.isSameHost(other: RelayItemId?): Boolean =
        this is GeoLocationId.Hostname && other == this
}
