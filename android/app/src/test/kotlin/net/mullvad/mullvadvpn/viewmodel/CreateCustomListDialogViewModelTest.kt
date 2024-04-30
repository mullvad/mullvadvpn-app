package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.usecase.customlists.CreateCustomListWithLocationsError
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
            val expectedResult: CustomListResult.Created = mockk()
            val customListName = "list"
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery {
                mockCustomListActionUseCase.performAction(any<CustomListAction.Create>())
            } returns expectedResult.right()
            every { expectedResult.locationNames } returns listOf("locationName")

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.createCustomList(customListName)
                val sideEffect = awaitItem()
                assertIs<CreateCustomListDialogSideEffect.ReturnWithResult>(sideEffect)
                assertEquals(expectedResult, sideEffect.result)
            }
        }

    @Test
    fun `when successfully creating a list without locations should emit with navigate to location screen`() =
        runTest {
            // Arrange
            val expectedResult: CustomListResult.Created = mockk()
            val customListName = "list"
            val createdId = CustomListId("1")
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery {
                mockCustomListActionUseCase.performAction(any<CustomListAction.Create>())
            } returns expectedResult.right()
            every { expectedResult.locationNames } returns emptyList()
            every { expectedResult.id } returns createdId

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.createCustomList(customListName)
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
        val expectedError =
            CreateCustomListWithLocationsError.Create(CreateCustomListError.CustomListAlreadyExists)
        val customListName = "list"
        val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.Create>())
        } returns expectedError.left()

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
            val expectedError =
                CreateCustomListWithLocationsError.Create(
                    CreateCustomListError.CustomListAlreadyExists
                )
            val customListName = "list"
            val viewModel = createViewModelWithLocationCode(GeoLocationId.Country("AB"))
            coEvery {
                mockCustomListActionUseCase.performAction(any<CustomListAction.Create>())
            } returns expectedError.left()

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
            locationCode = locationCode,
            customListActionUseCase = mockCustomListActionUseCase
        )
}
