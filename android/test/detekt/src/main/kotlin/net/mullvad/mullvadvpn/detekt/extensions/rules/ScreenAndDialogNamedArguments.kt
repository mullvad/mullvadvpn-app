package net.mullvad.mullvadvpn.detekt.extensions.rules

import io.gitlab.arturbosch.detekt.api.CodeSmell
import io.gitlab.arturbosch.detekt.api.Config
import io.gitlab.arturbosch.detekt.api.Debt
import io.gitlab.arturbosch.detekt.api.Entity
import io.gitlab.arturbosch.detekt.api.Issue
import io.gitlab.arturbosch.detekt.api.Rule
import io.gitlab.arturbosch.detekt.api.Severity
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtLambdaArgument

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
        val name = expression.calleeExpression?.text ?: return

        if (!isProbablyScreenOrDialog(name)) return

        val args =
            expression.valueArguments.let {
                val skipLast = it.lastOrNull() is KtLambdaArgument
                if (skipLast) it.dropLast(1) else it
            }

        val hasUnnamed = args.any { !it.isNamed() }
        if (hasUnnamed) {
            report(
                CodeSmell(
                    issue = issue,
                    entity = Entity.from(element = expression.originalElement, offset = 0),
                    message = "Call to composable `$name` must use only named arguments.",
                )
            )
        }
    }

    // We can't access the function declaration to see if this is a @Composable here.
    private fun isProbablyScreenOrDialog(name: String): Boolean =
        name[0].isUpperCase() && (name.endsWith("Screen") || name.endsWith("Dialog"))
}
