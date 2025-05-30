package net.mullvad.mullvadvpn.detekt.extensions.rules

import io.gitlab.arturbosch.detekt.api.CodeSmell
import io.gitlab.arturbosch.detekt.api.Config
import io.gitlab.arturbosch.detekt.api.Debt
import io.gitlab.arturbosch.detekt.api.Entity
import io.gitlab.arturbosch.detekt.api.Issue
import io.gitlab.arturbosch.detekt.api.Rule
import io.gitlab.arturbosch.detekt.api.Severity
import org.jetbrains.kotlin.psi.KtCallExpression

class ScreenAndDialogNamedArguments(config: Config) : Rule(config) {

    override val issue =
        Issue(
            javaClass.simpleName,
            Severity.CodeSmell,
            "This rule reports Screen and Dialog composable calls that do not exclusively use named arguments",
            Debt(mins = 1),
        )

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val name = expression.calleeExpression?.text ?: ""
        if (!name.endsWith("Screen") && !name.endsWith("Dialog")) return

        val hasUnnamed = expression.valueArguments.any { !it.isNamed() }
        if (hasUnnamed) {
            report(
                CodeSmell(
                    issue = issue,
                    entity = Entity.from(element = expression.originalElement, offset = 0),
                    message = "Call to composable $name must use only named arguments.",
                ),
            )
        }
    }
}
