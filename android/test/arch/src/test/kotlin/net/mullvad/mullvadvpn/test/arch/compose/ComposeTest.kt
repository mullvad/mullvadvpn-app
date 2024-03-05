package net.mullvad.mullvadvpn.test.arch.compose

import androidx.compose.runtime.Composable
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAllAnnotationsOf
import com.lemonappdev.konsist.api.verify.assertFalse
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class ComposeTest {
    @Test
    fun `ensure all app composables are in the compose package`() =
        allAppComposeFunctions().assertTrue {
            it.resideInPackage("net.mullvad.mullvadvpn.compose..")
        }

    @Test
    fun `ensure we don't use collectAsState`() =
        Konsist.scopeFromProduction("app").imports.assertFalse {
            it.name == "androidx.compose.runtime.collectAsState"
        }

    @Test
    fun `ensure all composables do not refer to state as uiState`() =
        allAppComposeFunctions().assertFalse { it.hasParameter { it.name == "uiState" } }

    private fun allAppComposeFunctions() =
        Konsist.scopeFromProduction("app").functions().withAllAnnotationsOf(Composable::class)
}
