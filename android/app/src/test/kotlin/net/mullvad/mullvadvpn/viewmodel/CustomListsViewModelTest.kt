package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CustomListsViewModelTest {
    private val mockRelayListUseCase: RelayListUseCase = mockk(relaxed = true)
    private val mockCustomListsActionUseCase: CustomListActionUseCase = mockk(relaxed = true)

    @Test
    fun `given custom list from relay list use case should be in state`() = runTest {
        // Arrange
        val customLists: List<RelayItem.CustomList> = mockk()
        val expectedState = CustomListsUiState.Content(customLists)
        every { mockRelayListUseCase.customLists() } returns flowOf(customLists)
        val viewModel = createViewModel()

        // Act, Assert
        viewModel.uiState.test { assertEquals(expectedState, awaitItem()) }
    }

    @Test
    fun `undo delete action should call custom list use case`() = runTest {
        // Arrange
        val viewModel = createViewModel()
        val action: CustomListAction.Create = mockk()

        // Act
        viewModel.undoDeleteCustomList(action)

        // Assert
        coVerify { mockCustomListsActionUseCase.performAction(action) }
    }

    private fun createViewModel() =
        CustomListsViewModel(
            relayListUseCase = mockRelayListUseCase,
            customListActionUseCase = mockCustomListsActionUseCase
        )
}
