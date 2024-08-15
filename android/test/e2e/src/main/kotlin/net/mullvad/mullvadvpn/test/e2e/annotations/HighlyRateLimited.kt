package net.mullvad.mullvadvpn.test.e2e.annotations

import androidx.test.platform.app.InstrumentationRegistry
import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext
import net.mullvad.mullvadvpn.test.e2e.constant.IGNORE_HIGHLY_RATE_LIMITED
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument

/**
 * Annotation for tests making use of API endpoints/requests that are highly rate limited such as failed login requests.
 */
@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(HighlyRateLimited.ShouldRunWhenSeverelyAffectedByRateLimiting::class)
annotation class HighlyRateLimited {
    class ShouldRunWhenSeverelyAffectedByRateLimiting: ExecutionCondition {
        override fun evaluateExecutionCondition(context: ExtensionContext?): ConditionEvaluationResult {
            val ignoreHighlyRateLimited = InstrumentationRegistry.getArguments().getRequiredArgument(
                IGNORE_HIGHLY_RATE_LIMITED).toBoolean()

            if (ignoreHighlyRateLimited) {
                return ConditionEvaluationResult.disabled("Skipping test because this run is configured to skip tests that are highly affected by rate limiting")
            } else {
                return ConditionEvaluationResult.enabled("Running test highly affected by rate limiting")
            }
        }
    }
}
