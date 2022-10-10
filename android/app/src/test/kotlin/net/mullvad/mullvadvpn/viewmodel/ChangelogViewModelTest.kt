package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import junit.framework.Assert
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import org.junit.After
import org.junit.Before
import org.junit.Test

class ChangelogViewModelTest {

    @MockK
    private lateinit var mockedChangelogRepository: ChangelogRepository

    private lateinit var viewModel: ChangelogViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        every {
            mockedChangelogRepository.setVersionCodeOfMostRecentChangelogShowed(any())
        } just Runs
        viewModel = ChangelogViewModel(mockedChangelogRepository, 1, false)
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testInitialState() = runBlockingTest {
        // Arrange, Act, Assert
        viewModel.changelogDialogUiState.test {
            Assert.assertEquals(ChangelogDialogUiState.Hide, awaitItem())
        }
    }

    @Test
    fun testShowAndDismissChangelogDialog() = runBlockingTest {
        viewModel.changelogDialogUiState.test {
            // Arrange
            val fakeList = listOf("test")
            every {
                mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed()
            } returns -1
            every { mockedChangelogRepository.getLastVersionChanges() } returns fakeList

            // Assert initial ui state
            assertEquals(ChangelogDialogUiState.Hide, awaitItem())

            // Refresh and verify that the dialog should be shown
            viewModel.refreshChangelogDialogUiState()
            assertEquals(ChangelogDialogUiState.Show(fakeList), awaitItem())

            // Dismiss dialog and verify that the dialog should be hidden
            viewModel.dismissChangelogDialog()
            assertEquals(ChangelogDialogUiState.Hide, awaitItem())
            verify { mockedChangelogRepository.setVersionCodeOfMostRecentChangelogShowed(1) }
        }
    }

    @Test
    fun testShowCaseChangelogWithEmptyListDialog() = runBlockingTest {
        viewModel.changelogDialogUiState.test {
            // Arrange
            val fakeEmptyList = emptyList<String>()
            every {
                mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed()
            } returns -1
            every { mockedChangelogRepository.getLastVersionChanges() } returns fakeEmptyList

            // Assert initial ui state
            assertEquals(ChangelogDialogUiState.Hide, awaitItem())

            // Refresh and verify that the Ui state remain same due list being empty
            viewModel.refreshChangelogDialogUiState()
            expectNoEvents()
        }
    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
    }
}
