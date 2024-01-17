package net.mullvad.mullvadvpn.test.common.annotation

import org.junit.jupiter.api.extension.ConditionEvaluationResult
import org.junit.jupiter.api.extension.ExecutionCondition
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(SkipForFlavors.FlavorCondition::class)
annotation class SkipForFlavors(val currentFlavor: String, vararg val skipForFlavors: String) {
    class FlavorCondition : ExecutionCondition {
        override fun evaluateExecutionCondition(p0: ExtensionContext?): ConditionEvaluationResult {
            val annotation = p0?.element?.get()?.getAnnotation(SkipForFlavors::class.java)
            return if (annotation?.skipForFlavors?.contains(annotation.currentFlavor) == true) {
                ConditionEvaluationResult.disabled(
                    "Skipping test for flavor: ${annotation.currentFlavor}"
                )
            } else {
                ConditionEvaluationResult.enabled(
                    "Running test for flavor: ${annotation?.currentFlavor}"
                )
            }
        }
    }
}
