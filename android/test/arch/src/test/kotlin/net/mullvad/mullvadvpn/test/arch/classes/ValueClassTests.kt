package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withValueModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.Test

class ValueClassTests {
    @Test
    fun `ensure value classes property is named value`() {
        Konsist.scopeFromProject().classes(includeNested = true).withValueModifier().assertTrue {
            it.properties(includeNested = false, includeLocal = false).firstOrNull { property ->
                property.name == "value"
            } != null
        }
    }
}
