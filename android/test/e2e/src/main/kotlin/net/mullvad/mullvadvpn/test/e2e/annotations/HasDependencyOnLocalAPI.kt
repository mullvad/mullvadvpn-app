package net.mullvad.mullvadvpn.test.e2e.annotations

import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

/**
 * Annotation for tests making use of local APIs such as the firewall or packet capture APIs, which
 * can only run in the office environment.
 */
@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(HasDependencyOnLocalAPI.ShouldRunWhenHaveAccessToLocalAPI::class)
annotation class HasDependencyOnLocalAPI {
    class ShouldRunWhenHaveAccessToLocalAPI : ExecutionCondition {
        override fun evaluateExecutionCondition(
            context: ExtensionContext?
        ): ConditionEvaluationResult {
            val enable = BuildConfig.ENABLE_ACCESS_TO_LOCAL_API_TESTS.toBoolean()

            return if (enable) {
                ConditionEvaluationResult.enabled(
                    "Running test which requires access to local APIs."
                )
            } else {
                ConditionEvaluationResult.disabled(
                    "Skipping test which requires access to local APIs."
                )
            }
        }
    }
}
