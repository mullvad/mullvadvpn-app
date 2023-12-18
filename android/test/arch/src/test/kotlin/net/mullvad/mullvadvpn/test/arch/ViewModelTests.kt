package net.mullvad.mullvadvpn.test.arch

import androidx.lifecycle.ViewModel
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.functions
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withPublicOrDefaultModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.ext.list.withAllParentsOf
import com.lemonappdev.konsist.api.verify.assert
import com.lemonappdev.konsist.api.verify.assertNot
import org.junit.jupiter.api.Test

class ViewModelTests {
    @Test
    fun `ensure view models have view model suffix`() =
        allViewModels().assert { it.name.endsWith("ViewModel") }

    // The purpose of this check is to both keep the naming consistent and also to avoid exposing
    // properties that shouldn't be exposed.
    @Test
    fun `ensure public properties use permitted names`() =
        allViewModels().properties(includeNested = false).withPublicOrDefaultModifier().assert {
            property ->
            property.name == "uiState" || property.name == "uiSideEffect"
        }

    @Test
    fun `ensure public functions have no return type`() =
        allViewModels().functions().withPublicOrDefaultModifier().assertNot { function ->
            function.hasReturnType()
        }

    private fun allViewModels() =
        Konsist.scopeFromProject().classes().withAllParentsOf(ViewModel::class)
}
