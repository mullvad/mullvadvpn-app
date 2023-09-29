package net.mullvad.mullvadvpn.test.arch

import androidx.lifecycle.ViewModel
import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAllParentsOf
import com.lemonappdev.konsist.api.verify.assert
import org.junit.Test

class ViewModelTests {
    @Test
    fun ensureViewModelsHaveViewModelSuffix() {
        Konsist.scopeFromProject().classes().withAllParentsOf(ViewModel::class).assert {
            it.name.endsWith("ViewModel")
        }
    }
}
