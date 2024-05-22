package net.mullvad.mullvadvpn.usecase

import arrow.core.right
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsException
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class CustomListActionUseCaseTest {
    private val mockCustomListsRepository: CustomListsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val customListActionUseCase =
        CustomListActionUseCase(
            customListsRepository = mockCustomListsRepository,
            relayListRepository = mockRelayListRepository
        )

    @BeforeEach
    fun setup() {
        mockkStatic(RELAY_LIST_EXTENSIONS)
    }

    @Test
    fun `create action should return success when ok`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val locationId = GeoLocationId.Country("se")
        val locationName = "Acklaba"
        val createdId = CustomListId("1")
        val action = CustomListAction.Create(name = name, locations = listOf(locationId))
        val expectedResult =
            Result.success(
                CustomListResult.Created(
                    id = createdId,
                    name = name,
                    locationNames = listOf(locationName),
                    undo = action.not(createdId)
                )
            )
        val relayItem =
            RelayItem.Location.Country(
                name = locationName,
                id = locationId,
                expanded = false,
                cities = emptyList()
            )
        val mockLocations: List<RelayItem.Location.Country> = listOf(relayItem)
        coEvery { mockCustomListsRepository.createCustomList(name) } returns createdId.right()
        coEvery {
            mockCustomListsRepository.updateCustomListLocations(createdId, listOf(locationId))
        } returns Unit.right()

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `create action should return error when name already exists`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val locationCode = "AB"
        val action = CustomListAction.Create(name = name, locations = listOf(locationCode))
        val expectedError = CreateCustomListError.CustomListExists
        coEvery { mockCustomListsRepository.createCustomList(name) } returns
            CreateCustomListResult.Error(CreateCustomListError.CustomListExists)

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertIs<Result<CustomListsException>>(result)
        val exception = result.exceptionOrNull()
        assertIs<CustomListsException>(exception)
        assertEquals(expectedError, exception.error)
    }

    @Test
    fun `rename action should return success when ok`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val newName = CustomListName.fromString("test2")
        val customListId = "1"
        val action = CustomListAction.Rename(id = customListId, name = name, newName = newName)
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
    fun `rename action should return error when name already exists`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val newName = CustomListName.fromString("test2")
        val customListId = "1"
        val action = CustomListAction.Rename(id = customListId, name = name, newName = newName)
        val expectedError = CreateCustomListError.CustomListExists
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
    fun `delete action should return successful with deleted list`() = runTest {
        // Arrange
        val mockCustomList: CustomList = mockk()
        val mockLocation: GeoLocationId.Country = mockk()
        val mockLocations: ArrayList<GeoLocationId> = arrayListOf(mockLocation)
        val name = CustomListName.fromString("test")
        val customListId = "1"
        val locationCode = "AB"
        val action = CustomListAction.Delete(id = customListId)
        val expectedResult =
            Result.success(
                CustomListResult.Deleted(
                    undo = action.not(name = name, locations = listOf(locationCode))
                )
            )
        every { mockCustomList.locations } returns mockLocations
        every { mockCustomList.name } returns name.value
        every { mockLocation.countryCode } returns locationCode
        coEvery { mockCustomListsRepository.deleteCustomList(id = customListId) } returns true
        every { mockCustomListsRepository.getCustomListById(customListId) } returns mockCustomList

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `update locations action should return success with changed locations`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val oldLocationCodes = listOf("AB", "CD")
        val newLocationCodes = listOf("EF", "GH")
        val oldLocations: ArrayList<GeoLocationId> =
            arrayListOf(GeoLocationId.Country("AB"), GeoLocationId.Country("CD"))
        val customListId = "1"
        val customList = CustomList(id = customListId, name = name.value, locations = oldLocations)
        val action =
            CustomListAction.UpdateLocations(id = customListId, locations = newLocationCodes)
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
