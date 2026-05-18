package net.mullvad.mullvadvpn.detekt.extensions.rules

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtLambdaArgument

class ScreenAndDialogNamedArguments(config: Config) :
    Rule(
        config,
        description =
            "This rule reports Screen and Dialog composable calls that do not exclusively use named arguments",
    ) {
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
                Finding(
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
