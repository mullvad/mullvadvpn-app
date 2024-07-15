package net.mullvad.mullvadvpn.test.arch.compose

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAllAnnotationsOf
import com.lemonappdev.konsist.api.verify.assertTrue
import com.ramcosta.composedestinations.annotation.Destination
import org.junit.jupiter.api.Test

class ComposeDestinationsTest {
    @Test
    fun `ensure all destinations functions does not have invalid suffix`() =
        allDestinationsFunctions().assertTrue { destinationFunction ->
            DESTINATION_SUFFIXES.none { suffix -> destinationFunction.name.endsWith(suffix) }
        }

    @Test
    fun `ensure all files that contains destinations ends with a valid suffix`() =
        allDestinationsFunctions().assertTrue { destinationFunction ->
            DESTINATION_SUFFIXES.any { destinationFunction.containingFile.name.endsWith(it) }
        }

    private fun allDestinationsFunctions() =
        Konsist.scopeFromProduction("app").functions().withAllAnnotationsOf(Destination::class)

    companion object {
        private val DESTINATION_SUFFIXES = listOf("Screen", "Dialog")
    }
}
