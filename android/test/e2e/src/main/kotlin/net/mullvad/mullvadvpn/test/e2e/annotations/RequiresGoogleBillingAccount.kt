package net.mullvad.mullvadvpn.test.e2e.annotations

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.isBillingEnabled
import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

/**
 * Annotation for tests making use of a google billing test account in order to perform purchases
 */
@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(RequiresGoogleBillingAccount.AccessToBillingTestAccount::class)
annotation class RequiresGoogleBillingAccount {
    class AccessToBillingTestAccount : ExecutionCondition {
        override fun evaluateExecutionCondition(
            context: ExtensionContext?
        ): ConditionEvaluationResult {

            val enable = InstrumentationRegistry.getArguments().isBillingEnabled()

            return if (enable) {
                ConditionEvaluationResult.enabled(
                    "Running test which requires access to billing test account."
                )
            } else {
                ConditionEvaluationResult.disabled(
                    "Skipping test which requires access to billing test account."
                )
            }
        }
    }
}
