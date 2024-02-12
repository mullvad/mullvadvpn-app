package net.mullvad.mullvadvpn.test.arch.compose

import androidx.compose.ui.tooling.preview.Preview
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAllAnnotationsOf
import com.lemonappdev.konsist.api.verify.assert
import org.junit.jupiter.api.Test

class ComposePreviewTest {
    @Test
    fun `ensure all preview functions are private`() =
        allPreviewFunctions().assert { it.hasPrivateModifier }

    @Test
    fun `ensure all preview functions are prefixed with 'Preview'`() =
        allPreviewFunctions().assert { it.name.startsWith("Preview") }

    private fun allPreviewFunctions() =
        Konsist.scopeFromProduction("app").functions().withAllAnnotationsOf(Preview::class)
}
