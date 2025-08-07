package net.mullvad.mullvadvpn.usecase

import arrow.core.Either
import arrow.core.left
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.core.right
import co.touchlab.kermit.Logger
import kotlin.collections.first
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class ModifyMultihopUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val customListsRepository: CustomListsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    suspend operator fun invoke(change: MultihopChange): Either<ModifyMultihopError, Unit> =
        either {
            ensure(change.item.active) { ModifyMultihopError.RelayItemInactive(change.item) }
            val changeId: RelayItemId =
                change.item.id.convertCustomListWithOnlyHostNameToHostName().bind()
            val other =
                when (change) {
                        is MultihopChange.Entry ->
                            settingsRepository.settingsUpdates.value.exit().bind()
                        is MultihopChange.Exit ->
                            settingsRepository.settingsUpdates.value.entry().bind()
                    }
                    .convertCustomListWithOnlyHostNameToHostName()
                    .bind()
            ensure(!changeId.isSameHost(other)) {
                when (change) {
                    is MultihopChange.Entry -> ModifyMultihopError.ExitSame(change.item)
                    is MultihopChange.Exit -> ModifyMultihopError.EntrySame(change.item)
                }
            }
            when (change) {
                    is MultihopChange.Entry ->
                        wireguardConstraintsRepository.setEntryLocation(change.item.id)
                    is MultihopChange.Exit ->
                        relayListRepository.updateSelectedRelayLocation(change.item.id)
                }
                .mapLeft {
                    Logger.e("Failed to update multihop: $it")
                    ModifyMultihopError.GenericError
                }
                .bind()
        }

    private fun Settings?.exit(): Either<ModifyMultihopError.GenericError, RelayItemId> =
        this?.relaySettings?.relayConstraints?.location?.getOrNull()?.right()
            ?: ModifyMultihopError.GenericError.left()

    private fun Settings?.entry(): Either<ModifyMultihopError.GenericError, RelayItemId> =
        this?.relaySettings
            ?.relayConstraints
            ?.wireguardConstraints
            ?.entryLocation
            ?.getOrNull()
            ?.right() ?: ModifyMultihopError.GenericError.left()

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

    private fun RelayItemId.isSameHost(other: RelayItemId): Boolean =
        this is GeoLocationId.Hostname && other == this
}

sealed class MultihopChange {
    abstract val item: RelayItem

    data class Entry(override val item: RelayItem) : MultihopChange()

    data class Exit(override val item: RelayItem) : MultihopChange()
}

sealed interface ModifyMultihopError {
    data class RelayItemInactive(val relayItem: RelayItem) : ModifyMultihopError

    data class EntrySame(val relayItem: RelayItem) : ModifyMultihopError

    data class ExitSame(val relayItem: RelayItem) : ModifyMultihopError

    data object GenericError : ModifyMultihopError
}
