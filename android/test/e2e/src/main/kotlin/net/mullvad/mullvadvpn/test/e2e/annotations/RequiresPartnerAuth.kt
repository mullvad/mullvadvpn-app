package net.mullvad.mullvadvpn.test.e2e.annotations

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.getPartnerAuth
import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

/** Annotation for tests requiring a partner api authentication. */
@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(RequiresPartnerAuth.HasPartnerAuth::class)
annotation class RequiresPartnerAuth {
    class HasPartnerAuth : ExecutionCondition {
        override fun evaluateExecutionCondition(
            context: ExtensionContext?
        ): ConditionEvaluationResult {

            val provided =
                InstrumentationRegistry.getArguments().getPartnerAuth()?.isNotEmpty() ?: false

            return if (provided) {
                ConditionEvaluationResult.enabled(
                    "Running test which requires partner authentication."
                )
            } else {
                ConditionEvaluationResult.disabled(
                    "Skipping test which requires partner authentication."
                )
            }
        }
    }
}
