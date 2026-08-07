package net.mullvad.mullvadvpn.lib.usecase

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.core.raise.ensureNotNull
import arrow.core.right
import co.touchlab.kermit.Logger
import kotlin.collections.first
import net.mullvad.mullvadvpn.lib.common.util.location
import net.mullvad.mullvadvpn.lib.common.util.multihopMode
import net.mullvad.mullvadvpn.lib.common.util.wireguardConstraints
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.map
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class ModifyMultihopUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val customListsRepository: CustomListsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    suspend operator fun invoke(change: MultihopChange): Either<ModifyMultihopError, Unit> =
        either {
            validate(
                    change = change,
                    settingsRepository = settingsRepository,
                    customListsRepository = customListsRepository,
                )
                .bind()

            when (change) {
                    is MultihopChange.Entry ->
                        wireguardConstraintsRepository.setEntryLocation(change.item.map { it.id })
                    is MultihopChange.Exit ->
                        relayListRepository.updateSelectedRelayLocation(change.item.id)
                }
                .mapLeft {
                    Logger.e("Failed to update multihop: $it")
                    ModifyMultihopError.GenericError
                }
                .bind()
        }
}

internal fun validate(
    change: MultihopChange,
    settingsRepository: SettingsRepository,
    customListsRepository: CustomListsRepository,
): Either<ModifyMultihopError, Unit> = either {
    val item = change.itemOrNull() ?: return Either.Right(Unit)

    ensure(item.active) { ModifyMultihopError.RelayItemInactive(item) }
    val changeId: RelayItemId =
        item.id.convertCustomListWithOnlyHostnameToHostname(customListsRepository).bind()
    val settings = settingsRepository.settingsUpdates.value
    ensureNotNull(settings) { ModifyMultihopError.GenericError }

    // If we are doing a when needed multihop, allow same relay for entry and exit.
    if (settings.multihopMode() == MultihopMode.ALWAYS) {
        val other =
            when (change) {
                    is MultihopChange.Entry -> settings.location().getOrNull()
                    is MultihopChange.Exit ->
                        settings.wireguardConstraints().entryLocation.getOrNull()
                }
                ?.convertCustomListWithOnlyHostnameToHostname(customListsRepository)
                ?.bind()
        ensure(!changeId.isSameHost(other)) { ModifyMultihopError.EntrySameAsExit(item) }
    }
}

private fun RelayItemId.convertCustomListWithOnlyHostnameToHostname(
    customListsRepository: CustomListsRepository
): Either<ModifyMultihopError.GenericError, RelayItemId> =
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

sealed interface MultihopChange {
    data class Entry(val item: Constraint<RelayItem>) : MultihopChange

    data class Exit(val item: RelayItem) : MultihopChange
}

// Returns the item of the change or null if this is a Constraint.Any.
fun MultihopChange.itemOrNull(): RelayItem? =
    when (this) {
        is MultihopChange.Entry -> item.getOrNull()
        is MultihopChange.Exit -> item
    }

sealed interface ModifyMultihopError {
    data class RelayItemInactive(val relayItem: RelayItem) : ModifyMultihopError

    data class EntrySameAsExit(val relayItem: RelayItem) : ModifyMultihopError

    data object GenericError : ModifyMultihopError
}
