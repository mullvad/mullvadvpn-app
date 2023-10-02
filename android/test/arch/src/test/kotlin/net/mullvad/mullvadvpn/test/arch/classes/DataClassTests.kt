package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.ext.list.modifierprovider.withDataModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assertNot
import net.mullvad.mullvadvpn.test.arch.extensions.projectScope
import org.junit.Test

class DataClasses {
    @Test
    fun `data classes use only immutable declarations`() {
        projectScope()
            .classes(includeNested = true)
            .withDataModifier()
            .properties(includeNested = false, includeLocal = false)
            .assertNot { it.hasVarModifier }
    }
}
