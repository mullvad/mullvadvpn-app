@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.annotation.StringRes
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.ui.resource.R

enum class GoodToKnowPage(@StringRes val title: Int, @StringRes val body: Int) {
    BLOCKED_CONTENT(
        R.string.good_to_know_blocked_content_title,
        R.string.good_to_know_blocked_content_body,
    ),
    EXCLUDING_APPS(
        R.string.good_to_know_excluding_apps_title,
        R.string.good_to_know_excluding_apps_body,
    );

    companion object {
        fun forStep(step: GoodToKnowStep): GoodToKnowPage =
            when (step) {
                GoodToKnowStep.GoodToKnow1 -> BLOCKED_CONTENT
                GoodToKnowStep.GoodToKnow2 -> EXCLUDING_APPS
            }
    }
}
