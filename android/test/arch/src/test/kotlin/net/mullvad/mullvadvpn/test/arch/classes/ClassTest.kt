package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class ClassTest {
    @Test
    fun `ensure companion object is last declaration in the class`() =
        Konsist.scopeFromProject().classes(includeNested = true).assertTrue {
            val companionObject =
                it.objects(includeNested = false).lastOrNull { obj -> obj.hasCompanionModifier }
            if (companionObject != null) {
                it.declarations(includeNested = false, includeLocal = false).last() ==
                    companionObject
            } else {
                true
            }
        }

    @Test
    fun `ensure test classes have 'Test' suffix`() =
        Konsist.scopeFromTest()
            .classes(includeNested = false)
            .filter {
                // Filter classes that are not tests (for example shadowed classes)
                it.packagee?.name?.startsWith("net.mullvad.mullvadvpn") ?: false
            }
            .assertTrue { it.hasNameEndingWith("Test") }

    @Test
    fun `ensure that all view model test classes are annotated with TestCoroutineRule`() =
        Konsist.scopeFromTest()
            .classes(includeNested = false)
            .filter {
                // Only include classes that are view model tests
                // We want to ignore the class "ViewModelTest" which contains konsist tests
                it.name.contains(".+ViewModelTest".toRegex())
            }
            .assertTrue {
                it.hasAnnotation { annotation ->
                    annotation.name == "ExtendWith" &&
                        annotation.arguments.any { argument ->
                            argument.value == "TestCoroutineRule::class"
                        }
                }
            }
}
