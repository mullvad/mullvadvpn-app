package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.dialog.CreateCustomListNavArgs
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.CustomListAlreadyExists
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.usecase.customlists.CreateWithLocationsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CreateCustomListDialogViewModelTest {
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()

    @Test
    fun `when successfully creating a list with locations should emit return with result side effect`() =
        runTest {
            // Arrange
            val mockCreated: Created = mockk()
            val mockUndo: CustomListAction.Delete = mockk()
            val customListName = CustomListName.fromString("list")
            val customListId = CustomListId("1")
            val locationNames = listOf("locationName")
            val expectedResult =
                CustomListActionResultData.Success.CreatedWithLocations(
                    customListName = customListName,
                    locationNames = locationNames,
                    undo = mockUndo,
                )
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery { mockCustomListActionUseCase(any<CustomListAction.Create>()) } returns
                mockCreated.right()
            every { mockCreated.locationNames } returns locationNames
            every { mockCreated.name } returns customListName
            every { mockCreated.id } returns customListId
            every { mockCreated.undo } returns mockUndo

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.createCustomList(customListName.value)
                val sideEffect = awaitItem()
                assertIs<CreateCustomListDialogSideEffect.ReturnWithResult>(sideEffect)
                assertEquals(expectedResult, sideEffect.result)
            }
        }

    @Test
    fun `when successfully creating a list without locations should emit with navigate to location screen`() =
        runTest {
            // Arrange
            val customListName = CustomListName.fromString("list")
            val createdId = CustomListId("1")
            val expectedResult =
                Created(
                    id = createdId,
                    name = customListName,
                    locationNames = emptyList(),
                    undo = CustomListAction.Delete(createdId),
                )
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery { mockCustomListActionUseCase(any<CustomListAction.Create>()) } returns
                expectedResult.right()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.createCustomList(customListName.value)
                val sideEffect = awaitItem()
                assertIs<CreateCustomListDialogSideEffect.NavigateToCustomListLocationsScreen>(
                    sideEffect
                )
                assertEquals(createdId, sideEffect.customListId)
            }
        }

    @Test
    fun `when failing to creating a list should update ui state with error`() = runTest {
        // Arrange
        val expectedError = CreateWithLocationsError.Create(CustomListAlreadyExists)
        val customListName = "list"
        val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
        coEvery { mockCustomListActionUseCase(any<CustomListAction.Create>()) } returns
            expectedError.left()

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            viewModel.createCustomList(customListName)
            assertEquals(expectedError, awaitItem().error)
        }
    }

    @Test
    fun `given error state when calling clear error then should update to state without error`() =
        runTest {
            // Arrange
            val expectedError = CreateWithLocationsError.Create(CustomListAlreadyExists)
            val customListName = "list"
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery { mockCustomListActionUseCase(any<CustomListAction.Create>()) } returns
                expectedError.left()

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                viewModel.createCustomList(customListName)
                assertEquals(expectedError, awaitItem().error) // Showing error
                viewModel.clearError()
                assertNull(awaitItem().error)
            }
        }

    private fun createViewModelWithLocationCode(locationCode: GeoLocationId) =
        CreateCustomListDialogViewModel(
            customListActionUseCase = mockCustomListActionUseCase,
            savedStateHandle =
                CreateCustomListNavArgs(locationCode = locationCode).toSavedStateHandle(),
        )
}
