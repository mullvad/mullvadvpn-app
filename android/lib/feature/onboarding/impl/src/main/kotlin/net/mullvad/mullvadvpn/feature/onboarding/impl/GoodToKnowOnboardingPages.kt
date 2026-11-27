@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.ui.resource.R

enum class GoodToKnowOnboardingPage(
    @StringRes val title: Int,
    @StringRes val body: Int,
    @DrawableRes val illustration: Int,
) {
    BLOCKED_CONTENT(
        R.string.good_to_know_blocked_content_title,
        R.string.good_to_know_blocked_content_body,
        R.drawable.onboarding_switch_location,
    ),
    EXCLUDING_APPS(
        R.string.good_to_know_excluding_apps_title,
        R.string.good_to_know_excluding_apps_body,
        R.drawable.onboarding_split_tunneling,
    );

    companion object {
        fun forStep(step: GoodToKnowStep): GoodToKnowOnboardingPage =
            when (step) {
                GoodToKnowStep.GoodToKnow1 -> BLOCKED_CONTENT
                GoodToKnowStep.GoodToKnow2 -> EXCLUDING_APPS
            }
    }
}
