package net.mullvad.mullvadvpn.test.e2e.annotations

import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

/**
 * Annotation for tests making use of API endpoints/requests that are highly rate limited such as
 * failed login requests.
 */
@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(HighlyRateLimited.ShouldRunWhenSeverelyAffectedByRateLimiting::class)
annotation class HighlyRateLimited {
    class ShouldRunWhenSeverelyAffectedByRateLimiting : ExecutionCondition {
        override fun evaluateExecutionCondition(
            context: ExtensionContext?
        ): ConditionEvaluationResult {
            val enableHighlyRateLimited =
                BuildConfig.ENABLE_HIGHLY_RATE_LIMITED_TESTS.toBoolean() ?: false

            return if (enableHighlyRateLimited) {
                ConditionEvaluationResult.enabled("Running test highly affected by rate limiting.")
            } else {
                ConditionEvaluationResult.disabled(
                    "Skipping test highly affected by rate limiting."
                )
            }
        }
    }
}
