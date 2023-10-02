package net.mullvad.mullvadvpn.test.arch.classes

import com.lemonappdev.konsist.api.ext.list.modifierprovider.withDataModifier
import com.lemonappdev.konsist.api.verify.assert
import net.mullvad.mullvadvpn.test.arch.extensions.projectScope
import org.junit.Test

class DataClasses {
    @Test
    fun `data classes use only immutable declarations`() {
        projectScope().classes(includeNested = true).withDataModifier().assert { classDeclaration ->
            classDeclaration
                .properties(includeNested = false)
                .filter { classDeclaration == it.containingDeclaration }
                .none { it.hasVarModifier }
        }
    }
}
