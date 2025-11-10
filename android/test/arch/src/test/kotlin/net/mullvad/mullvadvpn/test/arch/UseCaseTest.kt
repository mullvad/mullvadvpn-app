package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.functions
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withPublicOrDefaultModifier
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class UseCaseTest {
    @Test
    fun `ensure all use cases end with UseCase suffix`() =
        allUseCasesFiles().assertTrue { it.name.endsWith("UseCase") }

    @Test
    fun `ensure every use cases is public`() =
        allUseCases().assertTrue { it.hasPublicOrDefaultModifier }

    @Test
    fun `ensure every public functions method is named 'invoke' with operator modifier`() =
        allUseCases().functions().withPublicOrDefaultModifier().assertTrue {
            it.name == "invoke" && it.hasOperatorModifier
        }

    private fun allUseCasesFiles() =
        Konsist.scopeFromProduction().files.filter { it.resideInPath("..usecase..") }

    private fun allUseCases() =
        Konsist.scopeFromProduction()
            .classes()
            .filter { it.resideInPackage("..usecase..") }
            .filter { !it.hasPrivateModifier }
            .filter { it.parentInterfaces().none { parent -> parent.name == "PaymentUseCase" } }
}
