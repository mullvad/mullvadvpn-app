package net.mullvad.mullvadvpn.usecase

import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertIs
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.getRelayItemsByCodes
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsException
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class CustomListActionUseCaseTest {
    private val mockCustomListsRepository: CustomListsRepository = mockk()
    private val mockRelayListUseCase: RelayListUseCase = mockk()
    private val customListActionUseCase =
        CustomListActionUseCase(
            customListsRepository = mockCustomListsRepository,
            relayListUseCase = mockRelayListUseCase
        )

    @BeforeEach
    fun setup() {
        mockkStatic(RELAY_LIST_EXTENSIONS)
    }

    @Test
    fun `give action create when successful should return created result`() = runTest {
        // Arrange
        val name = "test"
        val locationCode = "AB"
        val locationName = "Acklaba"
        val createdId = "1"
        val action = CustomListAction.Create(name = name, locations = listOf(locationCode))
        val expectedResult =
            Result.success(
                CustomListResult.Created(
                    id = createdId,
                    name = name,
                    locationName = locationName,
                    undo = action.not(createdId)
                )
            )
        val relayItem =
            RelayItem.Country(
                name = locationName,
                code = locationCode,
                expanded = false,
                cities = emptyList()
            )
        val mockLocations: List<RelayItem.Country> = listOf(relayItem)
        coEvery { mockCustomListsRepository.createCustomList(name) } returns
            CreateCustomListResult.Ok(createdId)
        coEvery {
            mockCustomListsRepository.updateCustomListLocationsFromCodes(
                createdId,
                listOf(locationCode)
            )
        } returns UpdateCustomListResult.Ok
        coEvery { mockRelayListUseCase.relayList() } returns flowOf(mockLocations)
        every { mockLocations.getRelayItemsByCodes(listOf(locationCode)) } returns mockLocations

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `give action create when list name already exits should return error`() = runTest {
        // Arrange
        val name = "test"
        val locationCode = "AB"
        val action = CustomListAction.Create(name = name, locations = listOf(locationCode))
        val expectedError = CustomListsError.CustomListExists
        coEvery { mockCustomListsRepository.createCustomList(name) } returns
            CreateCustomListResult.Error(CustomListsError.CustomListExists)

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertIs<Result<CustomListsException>>(result)
        val exception = result.exceptionOrNull()
        assertIs<CustomListsException>(exception)
        assertEquals(expectedError, exception.error)
    }

    @Test
    fun `give action rename when successful should return rename result`() = runTest {
        // Arrange
        val name = "test"
        val newName = "test2"
        val customListId = "1"
        val action =
            CustomListAction.Rename(customListId = customListId, name = name, newName = newName)
        val expectedResult = Result.success(CustomListResult.Renamed(undo = action.not()))
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns UpdateCustomListResult.Ok

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `give action rename when list name already exists should return error`() = runTest {
        // Arrange
        val name = "test"
        val newName = "test2"
        val customListId = "1"
        val action =
            CustomListAction.Rename(customListId = customListId, name = name, newName = newName)
        val expectedError = CustomListsError.CustomListExists
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns UpdateCustomListResult.Error(expectedError)

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertIs<Result<CustomListsException>>(result)
        val exception = result.exceptionOrNull()
        assertIs<CustomListsException>(exception)
        assertEquals(expectedError, exception.error)
    }

    @Test
    fun `give action delete when successful should return delete result`() = runTest {
        // Arrange
        val mockCustomList: CustomList = mockk()
        val mockLocation: GeographicLocationConstraint.Country = mockk()
        val mockLocations: ArrayList<GeographicLocationConstraint> = arrayListOf(mockLocation)
        val name = "test"
        val customListId = "1"
        val locationCode = "AB"
        val action = CustomListAction.Delete(customListId = customListId)
        val expectedResult =
            Result.success(
                CustomListResult.Deleted(
                    undo = action.not(name = name, locations = listOf(locationCode))
                )
            )
        every { mockCustomList.locations } returns mockLocations
        every { mockCustomList.name } returns name
        every { mockLocation.countryCode } returns locationCode
        coEvery { mockCustomListsRepository.deleteCustomList(id = customListId) } returns true
        every { mockCustomListsRepository.getCustomListById(customListId) } returns mockCustomList

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `give action update locations when successful should return locations changed result`() =
        runTest {
            // Arrange
            val name = "test"
            val oldLocationCodes = listOf("AB", "CD")
            val newLocationCodes = listOf("EF", "GH")
            val oldLocations: ArrayList<GeographicLocationConstraint> =
                arrayListOf(
                    GeographicLocationConstraint.Country("AB"),
                    GeographicLocationConstraint.Country("CD")
                )
            val customListId = "1"
            val customList = CustomList(id = customListId, name = name, locations = oldLocations)
            val action =
                CustomListAction.UpdateLocations(
                    customListId = customListId,
                    locations = newLocationCodes
                )
            val expectedResult =
                Result.success(
                    CustomListResult.LocationsChanged(
                        name = name,
                        undo = action.not(locations = oldLocationCodes)
                    )
                )
            coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns customList

            coEvery {
                mockCustomListsRepository.updateCustomListLocationsFromCodes(
                    customListId,
                    newLocationCodes
                )
            } returns UpdateCustomListResult.Ok

            // Act
            val result = customListActionUseCase.performAction(action)

            // Assert
            assertEquals(expectedResult, result)
        }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
