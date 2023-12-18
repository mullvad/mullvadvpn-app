package net.mullvad.mullvadvpn.test.arch.compose

import androidx.compose.runtime.Composable
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAllAnnotationsOf
import com.lemonappdev.konsist.api.verify.assert
import org.junit.jupiter.api.Test

class ComposeTests {
    @Test
    fun `ensure all app composables are in the compose package`() =
        allAppComposeFunctions().assert { it.resideInPackage("net.mullvad.mullvadvpn.compose..") }

    private fun allAppComposeFunctions() =
        Konsist.scopeFromProduction("app").functions().withAllAnnotationsOf(Composable::class)
}
