package net.mullvad.mullvadvpn.feature.applisting.api

fun interface ResolveAppListingUseCase {
    operator fun invoke(): AppListingTarget
}
