package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withValueModifier
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class ValueClassTest {
    @Test
    fun `ensure value classes property is named value`() {
        Konsist.scopeFromProject().classes(includeNested = true).withValueModifier().assertTrue {
            it.properties(includeNested = false).firstOrNull { property ->
                property.name == "value"
            } != null
        }
    }
}
