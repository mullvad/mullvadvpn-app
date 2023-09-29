package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.verify.assert
import net.mullvad.mullvadvpn.test.arch.extensions.projectScope
import org.junit.Test

class ClassTests {
    @Test
    fun `companion object is last declaration in the class`() {
        projectScope().classes(includeNested = true).assert {
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
}
