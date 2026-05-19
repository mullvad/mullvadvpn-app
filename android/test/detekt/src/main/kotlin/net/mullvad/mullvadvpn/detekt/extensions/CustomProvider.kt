package net.mullvad.mullvadvpn.detekt.extensions

import dev.detekt.api.RuleName
import dev.detekt.api.RuleSet
import dev.detekt.api.RuleSetId
import dev.detekt.api.RuleSetProvider
import kotlin.collections.mapOf
import net.mullvad.mullvadvpn.detekt.extensions.rules.ScreenAndDialogNamedArguments

class CustomProvider : RuleSetProvider {

    override val ruleSetId: RuleSetId = RuleSetId("custom")

    override fun instance(): RuleSet =
        RuleSet(
            ruleSetId,
            mapOf(
                RuleName("ScreenAndDialogNamedArguments") to
                    { config ->
                        ScreenAndDialogNamedArguments(config)
                    }
            ),
        )
}
