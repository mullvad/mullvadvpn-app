package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.repository.AppChangesRepository
import org.junit.After
import org.junit.Before
import org.junit.Test

class ChangeLogViewModelTest {

    @MockK
    private lateinit var mockedAppChangesRepository: AppChangesRepository

    private lateinit var viewModel: ChangelogViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        every { mockedAppChangesRepository.getVersionCodeOfMostRecentChangelogShowed() } returns -1
        viewModel = ChangelogViewModel(mockedAppChangesRepository)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testUiStateWhenNeedToShowChangeLog() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.changelogDialogUiState.test {
            viewModel.refreshChangelogDialogUiState()
        }
    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
    }
}
