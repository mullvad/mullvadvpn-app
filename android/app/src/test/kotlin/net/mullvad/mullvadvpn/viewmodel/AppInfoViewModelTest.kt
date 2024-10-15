package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AppInfoViewModelTest {

    @MockK private lateinit var mockedChangelogRepository: ChangelogRepository

    private lateinit var viewModel: AppInfoViewModel

    private val buildVersion = BuildVersion("1.0", 10)

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        every { mockedChangelogRepository.setVersionCodeOfMostRecentChangelogShowed(any()) } just
            Runs
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `given up to date version code uiSideEffect should not emit`() = runTest {
        // Arrange
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns
            buildVersion.code
        viewModel = AppInfoViewModel(mockedChangelogRepository, buildVersion, false)

        // If we have the most up to date version code, we should not show the changelog dialog
        viewModel.uiSideEffect.test { expectNoEvents() }
    }

    @Test
    fun `given old version code uiSideEffect should emit ChangeLog`() = runTest {
        // Arrange
        val version = -1
        val changes = listOf("first change", "second change")
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns
            version
        every { mockedChangelogRepository.getLastVersionChanges() } returns changes

        viewModel = AppInfoViewModel(mockedChangelogRepository, buildVersion, false)
        // Given a new version with a change log we should return it
        viewModel.uiSideEffect.test {
            assertEquals(awaitItem(), Changelog(version = buildVersion.name, changes = changes))
        }
    }

    @Test
    fun `given old version code and empty change log uiSideEffect should not emit`() = runTest {
        // Arrange
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns -1
        every { mockedChangelogRepository.getLastVersionChanges() } returns emptyList()

        viewModel = AppInfoViewModel(mockedChangelogRepository, buildVersion, false)
        // Given a new version with a change log we should not return it
        viewModel.uiSideEffect.test { expectNoEvents() }
    }
}
