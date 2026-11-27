package net.mullvad.mullvadvpn.feature.onboarding.api

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class OnboardingNavKey(val step: OnboardingStep) : NavKey2

@Parcelize
sealed interface OnboardingStep : Parcelable {
    data object AddTime : OnboardingStep

    data object Notifications : OnboardingStep

    data object ReadyToConnect : OnboardingStep
}

@Parcelize
sealed interface GoodToKnowStep : OnboardingStep {
    data object GoodToKnow1 : GoodToKnowStep

    data object GoodToKnow2 : GoodToKnowStep
}

@Parcelize data object OnboardingConnectNavResult : NavResult
