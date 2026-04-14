package net.mullvad.mullvadvpn.feature.splittunneling.impl.applist

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class SplitTunnelingUseCase(
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val applicationsProvider: ApplicationsProvider,
    private val preferencesRepository: UserPreferencesRepository,
    private val dispatcher: CoroutineDispatcher,
) {
    operator fun invoke(): Flow<SplitApps> =
        combine(
                flow { emit(applicationsProvider.apps()) },
                splitTunnelingRepository.excludedApps,
                splitTunnelingRepository.splitTunnelingEnabled,
                preferencesRepository.showSystemAppsSplitTunneling(),
            ) { allApps, exclusions, splitTunnelingEnabled, showSystemApps ->
                val exclusions = if (splitTunnelingEnabled) exclusions else emptySet()
                SplitApps(
                    allApps =
                        if (showSystemApps) allApps
                        else allApps.filter { !it.isSystemApp || it.packageName in exclusions },
                    exclusions = exclusions,
                )
            }
            .flowOn(dispatcher)
}

data class SplitApps(private val allApps: List<AppData>, private val exclusions: Set<PackageName>) {
    val includedApps: List<AppData>
    val excludedApps: List<AppData>

    init {
        allApps
            .partition { appData -> exclusions.contains(appData.packageName) }
            .also { (exclusions, inclusions) ->
                includedApps = inclusions
                excludedApps = exclusions
            }
    }
}
