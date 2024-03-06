package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class EditCustomListViewModelTest {
    private val mockRelayListUseCase: RelayListUseCase = mockk(relaxed = true)

    @Test
    fun `given a custom list id that does not exists should return not found ui state`() = runTest {
        // Arrange
        val customListId = "2"
        val customList =
            RelayItem.CustomList(id = "1", name = "test", expanded = false, locations = emptyList())
        every { mockRelayListUseCase.customLists() } returns flowOf(listOf(customList))
        val viewModel = createViewModel(customListId)

        // Act, Assert
        viewModel.uiState.test {
            val item = awaitItem()
            assertIs<EditCustomListState.NotFound>(item)
        }
    }

    @Test
    fun `given a custom list id that exists should return content ui state`() = runTest {
        // Arrange
        val customListId = "1"
        val customList =
            RelayItem.CustomList(
                id = customListId,
                name = "test",
                expanded = false,
                locations = emptyList()
            )
        every { mockRelayListUseCase.customLists() } returns flowOf(listOf(customList))
        val viewModel = createViewModel(customListId)

        // Act, Assert
        viewModel.uiState.test {
            val item = awaitItem()
            assertIs<EditCustomListState.Content>(item)
            assertEquals(item.id, customList.id)
            assertEquals(item.name, customList.name)
            assertEquals(item.locations, customList.locations)
        }
    }

    private fun createViewModel(customListId: String) =
        EditCustomListViewModel(
            customListId = customListId,
            relayListUseCase = mockRelayListUseCase
        )
}
