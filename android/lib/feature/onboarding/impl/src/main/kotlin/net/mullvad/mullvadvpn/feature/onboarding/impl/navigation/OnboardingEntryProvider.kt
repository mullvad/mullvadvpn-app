package net.mullvad.mullvadvpn.feature.onboarding.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.feature.onboarding.impl.AddTime
import net.mullvad.mullvadvpn.feature.onboarding.impl.GoodToKnow1
import net.mullvad.mullvadvpn.feature.onboarding.impl.GoodToKnow2
import net.mullvad.mullvadvpn.feature.onboarding.impl.Notifications
import net.mullvad.mullvadvpn.feature.onboarding.impl.ReadyToConnect

fun EntryProviderScope<NavKey2>.onboardingEntry(navigator: Navigator) {
    entry<OnboardingNavKey>(
        metadata = slideInHorizontalTransition(),
    ) { navKey ->
        when (navKey.step) {
            OnboardingStep.AddTime -> AddTime(navKey = navKey, navigator = navigator)
            OnboardingStep.Notifications -> Notifications(navigator = navigator)
            GoodToKnowStep.GoodToKnow1 -> GoodToKnow1(navigator = navigator)
            GoodToKnowStep.GoodToKnow2 -> GoodToKnow2(navigator = navigator)
            OnboardingStep.ReadyToConnect -> ReadyToConnect(navigator = navigator)
        }
    }
}
