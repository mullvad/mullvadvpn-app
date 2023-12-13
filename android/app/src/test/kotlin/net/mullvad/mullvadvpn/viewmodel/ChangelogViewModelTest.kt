package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertNotNull
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ChangelogViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @MockK private lateinit var mockedChangelogRepository: ChangelogRepository

    private lateinit var viewModel: ChangelogViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(EVENT_NOTIFIER_EXTENSION_CLASS)
        every { mockedChangelogRepository.setVersionCodeOfMostRecentChangelogShowed(any()) } just
            Runs
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun testUpToDateVersionCodeShouldNotEmitChangelog() = runTest {
        // Arrange
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns
                buildVersionCode
        viewModel = ChangelogViewModel(mockedChangelogRepository, buildVersionCode, false)

        // If we have the most up to date version code, we should not show the changelog dialog
        viewModel.uiSideEffect.test { expectNoEvents() }
    }

    @Test
    fun testNotUpToDateVersionCodeShouldEmitChangelog() = runTest {
        // Arrange
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns -1
        every { mockedChangelogRepository.getLastVersionChanges() } returns listOf("bla", "bla")

        viewModel = ChangelogViewModel(mockedChangelogRepository, buildVersionCode, false)
        // Given a new version with a change log we should return it
        viewModel.uiSideEffect.test { assertNotNull(awaitItem()) }
    }

    @Test
    fun testEmptyChangelogShouldNotEmitChangelog() = runTest {
        // Arrange
        every { mockedChangelogRepository.getVersionCodeOfMostRecentChangelogShowed() } returns -1
        every { mockedChangelogRepository.getLastVersionChanges() } returns emptyList()

        viewModel = ChangelogViewModel(mockedChangelogRepository, buildVersionCode, false)
        // Given a new version with a change log we should not return it
        viewModel.uiSideEffect.test { expectNoEvents() }
    }

    companion object {
        private const val EVENT_NOTIFIER_EXTENSION_CLASS =
            "net.mullvad.talpid.util.EventNotifierExtensionsKt"
        private const val buildVersionCode = 10
    }
}
