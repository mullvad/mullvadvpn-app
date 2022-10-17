package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.repository.AppChangesRepository
import net.mullvad.mullvadvpn.repository.ChangeLogState
import org.junit.After
import org.junit.Before
import org.junit.Test

class ChangeLogViewModelTest {

    @MockK
    private lateinit var mockedAppChangesRepository: AppChangesRepository

    private lateinit var viewModel: AppChangesViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        every { mockedAppChangesRepository.shouldShowLastChanges() } returns true
        every { mockedAppChangesRepository.resetShouldShowLastChanges() } returns Unit
        viewModel = AppChangesViewModel(mockedAppChangesRepository)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testUiStateWhenNeedToShowChangeLog() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.changeLogState.test {
            viewModel.resetShouldShowChanges()
            assertEquals(ChangeLogState.ShouldShow, awaitItem())
        }
    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
    }
}
