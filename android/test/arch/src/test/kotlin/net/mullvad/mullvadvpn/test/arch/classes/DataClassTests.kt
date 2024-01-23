package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withDataModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assertNot
import org.junit.jupiter.api.Test

class DataClassTests {
    @Test
    fun `ensure data classes only use immutable properties`() =
        Konsist.scopeFromProject()
            .classes(includeNested = true)
            .withDataModifier()
            .properties(includeNested = false, includeLocal = false)
            .assertNot { it.hasVarModifier }
}
