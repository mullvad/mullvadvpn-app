package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.dialog.EditCustomListNameNavArgs
import net.mullvad.mullvadvpn.compose.state.EditCustomListUiState
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
        val customListId = CustomListId("2")
        val name = CustomListName.fromString("test")
        val customList = CustomList(id = CustomListId("1"), name = name, locations = emptyList())
        every { mockCustomListsRepository.customLists } returns MutableStateFlow(listOf(customList))
        val viewModel = createViewModel(customListId, name)

        // Act, Assert
        viewModel.uiState.test {
            val item = awaitItem()
            assertIs<EditCustomListUiState.NotFound>(item)
        }
    }

    @Test
    fun `given a custom list id that exists should return content ui state`() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val name = CustomListName.fromString("test")
        val customList = CustomList(id = customListId, name = name, locations = emptyList())
        every { mockCustomListsRepository.customLists } returns MutableStateFlow(listOf(customList))
        val viewModel = createViewModel(customListId, name)

        // Act, Assert
        viewModel.uiState.test {
            val item = awaitItem()
            assertIs<EditCustomListUiState.Content>(item)
            assertEquals(item.id, customList.id)
            assertEquals(item.name, customList.name)
            assertEquals(item.locations, customList.locations)
        }
    }

    private fun createViewModel(customListId: CustomListId, initialName: CustomListName) =
        EditCustomListViewModel(
            customListsRepository = mockCustomListsRepository,
            savedStateHandle =
                EditCustomListNameNavArgs(customListId = customListId, initialName = initialName)
                    .toSavedStateHandle(),
        )
}
