package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.verify.assert
import org.junit.jupiter.api.Test

class ClassTests {
    @Test
    fun `ensure companion object is last declaration in the class`() =
        Konsist.scopeFromProject().classes(includeNested = true).assert {
            val companionObject =
                it.objects(includeNested = false).lastOrNull { obj -> obj.hasCompanionModifier }
            if (companionObject != null) {
                it.declarations(includeNested = false, includeLocal = false).last() ==
                    companionObject
            } else {
                true
            }
        }
}
