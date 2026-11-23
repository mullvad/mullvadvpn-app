package net.mullvad.mullvadvpn.feature.onboarding.impl

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ReadyToConnectViewModelTest {

    private val mockSelectedLocationTitleUseCase: SelectedLocationTitleUseCase = mockk()
    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)

    @Test
    fun `uiState should expose only the selected location`() = runTest {
        // Arrange
        val selectedLocation = MutableStateFlow<String?>(null)
        every { mockSelectedLocationTitleUseCase() } returns selectedLocation
        val viewModel =
            ReadyToConnectViewModel(
                selectedLocationTitleUseCase = mockSelectedLocationTitleUseCase,
                userPreferencesRepository = mockPreferencesRepository,
            )

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(null, awaitItem().selectedLocationTitle)
            selectedLocation.value = "Sweden"
            assertEquals("Sweden", awaitItem().selectedLocationTitle)
            cancelAndIgnoreRemainingEvents()
        }
    }
}
