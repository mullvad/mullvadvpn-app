package net.mullvad.mullvadvpn.test.arch

import androidx.lifecycle.ViewModel
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withPublicOrDefaultModifier
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.ext.list.withAllParentsOf
import com.lemonappdev.konsist.api.verify.assert
import org.junit.Test

class ViewModelTests {
    @Test
    fun ensureViewModelsHaveViewModelSuffix() {
        allViewModels().assert { it.name.endsWith("ViewModel") }
    }

    // The purpose of this check is to both keep the naming consistent and also to avoid exposing
    // properties that shouldn't be exposed.
    @Test
    fun ensurePublicPropertiesUsePermittedNames() {
        allViewModels().properties().withPublicOrDefaultModifier().assert { property ->
            permittedPublicPropertyNames.contains(property.name)
        }
    }

    private fun allViewModels() =
        Konsist.scopeFromProject().classes().withAllParentsOf(ViewModel::class)

    companion object {
        // TODO: The goal is to reduce this list to only "uiState" and "uiAction".
        private val permittedPublicPropertyNames =
            listOf(
                "uiState",
                "viewActions",
                "toastMessages",
                "uiCloseAction",
                "enterTransitionEndAction",
                "accountToken"
            )
    }
}
