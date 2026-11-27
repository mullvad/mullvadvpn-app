package net.mullvad.mullvadvpn.feature.onboarding.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.feature.onboarding.impl.AddTimeOnboarding
import net.mullvad.mullvadvpn.feature.onboarding.impl.GoodToKnowOnboarding1
import net.mullvad.mullvadvpn.feature.onboarding.impl.GoodToKnowOnboarding2
import net.mullvad.mullvadvpn.feature.onboarding.impl.NotificationsOnboarding
import net.mullvad.mullvadvpn.feature.onboarding.impl.ReadyToConnectOnboarding

fun EntryProviderScope<NavKey2>.onboardingEntry(navigator: Navigator) {
    entry<OnboardingNavKey>(
        metadata = slideInHorizontalTransition(),
    ) { navKey ->
        when (navKey.step) {
            OnboardingStep.AddTime -> AddTimeOnboarding(navKey = navKey, navigator = navigator)
            OnboardingStep.Notifications -> NotificationsOnboarding(navigator = navigator)
            GoodToKnowStep.GoodToKnow1 -> GoodToKnowOnboarding1(navigator = navigator)
            GoodToKnowStep.GoodToKnow2 -> GoodToKnowOnboarding2(navigator = navigator)
            OnboardingStep.ReadyToConnect -> ReadyToConnectOnboarding(navigator = navigator)
        }
    }
}
