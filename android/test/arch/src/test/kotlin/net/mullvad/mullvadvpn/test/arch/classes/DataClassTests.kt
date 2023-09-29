package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.ext.list.modifierprovider.withDataModifier
import com.lemonappdev.konsist.api.verify.assert
import net.mullvad.mullvadvpn.test.arch.extensions.projectScope
import org.junit.Ignore
import org.junit.Test

class DataClasses {
    @Ignore("Code needs clean up")
    @Test
    fun `data classes use only immutable parameters`() {
        projectScope().classes(includeNested = true).withDataModifier().assert {
            it.properties(includeNested = true).all { property -> property.hasValModifier }
        }
    }
}
