package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withDataModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assertFalse
import org.junit.jupiter.api.Test

class DataClassTest {
    @Test
    fun `ensure data classes only use immutable properties`() =
        Konsist.scopeFromProject()
            .classes(includeNested = true)
            .withDataModifier()
            // If includeNested is set to true the test would fail on properties of nested classes
            // of data classes, even if those classes are not used as a property of the data class.
            // For example:
            // data class Immutable(val a: String) {
            //     class Mutable {
            //         var b: String = ""
            //     }
            // }
            // would fail because the nested class Mutable has a var property.
            .properties(includeNested = false)
            .assertFalse { it.isVar }
}
