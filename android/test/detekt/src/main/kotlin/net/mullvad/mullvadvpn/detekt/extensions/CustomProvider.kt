package net.mullvad.mullvadvpn.detekt.extensions

import io.gitlab.arturbosch.detekt.api.Config
import io.gitlab.arturbosch.detekt.api.RuleSet
import io.gitlab.arturbosch.detekt.api.RuleSetProvider
import net.mullvad.mullvadvpn.detekt.extensions.rules.ScreenAndDialogNamedArguments

class CustomProvider : RuleSetProvider {

    override val ruleSetId: String = "custom"

    override fun instance(config: Config): RuleSet =
        RuleSet(ruleSetId, listOf(ScreenAndDialogNamedArguments(config)))
}
