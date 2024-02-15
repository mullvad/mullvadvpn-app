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
        Konsist.scopeFromTest().classes(includeNested = false).assertTrue {
            it.hasNameEndingWith("Test")
        }
}
