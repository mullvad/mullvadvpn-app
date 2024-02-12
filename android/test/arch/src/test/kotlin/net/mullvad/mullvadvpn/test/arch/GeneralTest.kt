package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assert
import com.lemonappdev.konsist.api.verify.assertNot
import org.junit.jupiter.api.Test

class GeneralTest {
    @Test
    fun `ensure package name must match file path`() =
        Konsist.scopeFromProject().packages.assert { it.hasMatchingPath }

    @Test
    fun `ensure no field should have 'm' prefix`() =
        Konsist.scopeFromProject().classes().properties().assertNot {
            val secondCharacterIsUppercase = it.name.getOrNull(1)?.isUpperCase() ?: false
            it.name.startsWith('m') && secondCharacterIsUppercase
        }

    @Test
    fun `ensure no empty files allowed`() =
        Konsist.scopeFromProject().files.assertNot { it.text.isEmpty() }
}
