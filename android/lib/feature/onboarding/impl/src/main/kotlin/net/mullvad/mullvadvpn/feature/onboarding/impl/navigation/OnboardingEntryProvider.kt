package net.mullvad.mullvadvpn.feature.onboarding.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.impl.Onboarding

fun EntryProviderScope<NavKey2>.onboardingEntry(navigator: Navigator) {
    entry<OnboardingNavKey>(
        metadata = slideInHorizontalTransition(),
    ) { key ->
        Onboarding(navKey = key, navigator = navigator)
    }
}
