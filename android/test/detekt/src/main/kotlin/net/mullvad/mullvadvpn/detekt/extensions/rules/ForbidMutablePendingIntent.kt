package net.mullvad.mullvadvpn.detekt.extensions.rules

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.symbols.KaCallableSymbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.psi.KtSimpleNameExpression

class ForbidMutablePendingIntent(config: Config) :
    Rule(
        config = config,
        description = "PendingIntent.FLAG_MUTABLE is forbidden. Use FLAG_IMMUTABLE instead.",
    ),
    RequiresAnalysisApi {

    override fun visitSimpleNameExpression(expression: KtSimpleNameExpression) {
        super.visitSimpleNameExpression(expression)

        if (expression.text == "FLAG_MUTABLE") {
            analyze(expression) {
                val symbol = expression.mainReference.resolveToSymbol() as? KaCallableSymbol
                val fqName = symbol?.callableId?.asSingleFqName()?.asString()

                if (fqName == "android.app.PendingIntent.FLAG_MUTABLE") {
                    report(
                        Finding(
                            entity = Entity.from(expression),
                            message = description,
                        )
                    )
                }
            }
        }
    }
}
