package net.mullvad.mullvadvpn.test.e2e.annotations

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.ENABLE_HIGHLY_RATE_LIMITED
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
                InstrumentationRegistry.getArguments()
                    .getString(ENABLE_HIGHLY_RATE_LIMITED)
                    ?.toBoolean() ?: false

            if (enableHighlyRateLimited) {
                return ConditionEvaluationResult.enabled(
                    "Running test highly affected by rate limiting."
                )
            } else {
                return ConditionEvaluationResult.disabled(
                    "Skipping test highly affected by rate limiting."
                )
            }
        }
    }
}
