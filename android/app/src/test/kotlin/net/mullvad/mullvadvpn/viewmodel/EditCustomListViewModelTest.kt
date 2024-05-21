package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class EditCustomListViewModelTest {
    private val mockCustomListsRepository: CustomListsRepository = mockk(relaxed = true)

    @Test
    fun `given a custom list id that does not exists should return not found ui state`() = runTest {
        // Arrange
        val customListId = net.mullvad.mullvadvpn.lib.model.CustomListId("2")
        val customList =
            CustomList(
                id = net.mullvad.mullvadvpn.lib.model.CustomListId("1"),
                name = CustomListName.fromString("test"),
                locations = emptyList()
            )
        every { mockCustomListsRepository.customLists } returns MutableStateFlow(listOf(customList))
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
        val customListId = net.mullvad.mullvadvpn.lib.model.CustomListId("1")
        val customList =
            CustomList(
                id = customListId,
                name = CustomListName.fromString("test"),
                locations = emptyList()
            )
        every { mockCustomListsRepository.customLists } returns MutableStateFlow(listOf(customList))
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

    private fun createViewModel(customListId: net.mullvad.mullvadvpn.lib.model.CustomListId) =
        EditCustomListViewModel(
            customListId = customListId,
            customListsRepository = mockCustomListsRepository
        )
}
