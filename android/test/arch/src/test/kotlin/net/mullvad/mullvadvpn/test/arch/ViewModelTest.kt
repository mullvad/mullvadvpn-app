package net.mullvad.mullvadvpn.test.arch

import androidx.lifecycle.ViewModel
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.functions
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withPublicOrDefaultModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.ext.list.withAllParentsOf
import com.lemonappdev.konsist.api.verify.assertFalse
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class ViewModelTest {
    @Test
    fun `ensure view models have view model suffix`() =
        allViewModels().assertTrue { it.name.endsWith("ViewModel") }

    // The purpose of this check is to both keep the naming consistent and also to avoid exposing
    // properties that shouldn't be exposed.
    @Test
    fun `ensure public properties use permitted names`() =
        allViewModels()
            .properties(includeNested = false)
            .withPublicOrDefaultModifier()
            .assertTrue { property ->
                property.name == "uiState" || property.name == "uiSideEffect"
            }

    @Test
    fun `ensure public functions have no return type`() =
        allViewModels().functions().withPublicOrDefaultModifier().assertFalse { function ->
            function.hasReturnType()
        }

    private fun allViewModels() =
        Konsist.scopeFromProject().classes().withAllParentsOf(ViewModel::class)
}
