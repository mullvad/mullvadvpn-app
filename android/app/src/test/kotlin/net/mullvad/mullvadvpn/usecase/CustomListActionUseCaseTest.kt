package net.mullvad.mullvadvpn.usecase

import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
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
        val locationId = GeoLocationId.Country("AB")
        val action = CustomListAction.Create(name = name, locations = listOf(locationId))
        val expectedError = CreateCustomListError.CustomListAlreadyExists
        coEvery { mockCustomListsRepository.createCustomList(name) } returns
            CreateCustomListError.CustomListAlreadyExists.left()

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedError, result)
    }

    @Test
    fun `rename action should return success when ok`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val newName = CustomListName.fromString("test2")
        val customListId = CustomListId("1")
        val action = CustomListAction.Rename(id = customListId, name = name, newName = newName)
        val expectedResult = Result.success(CustomListResult.Renamed(undo = action.not()))
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns Unit.right()

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
        val customListId = CustomListId("1")
        val action = CustomListAction.Rename(id = customListId, name = name, newName = newName)
        val expectedError = UpdateCustomListError.NameAlreadyExists(newName.value)
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns expectedError.left()

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedError, result)
    }

    @Test
    fun `delete action should return successful with deleted list`() = runTest {
        // Arrange
        val mockCustomList: CustomList = mockk()
        val mockLocation: GeoLocationId.Country = mockk()
        val mockLocations: List<GeoLocationId> = listOf(mockLocation)
        val name = CustomListName.fromString("test")
        val customListId = CustomListId("1")
        val location = GeoLocationId.Country("AB")
        val action = CustomListAction.Delete(id = customListId)
        val expectedResult =
            Result.success(
                CustomListResult.Deleted(
                    undo = action.not(name = name, locations = listOf(location))
                )
            )
        every { mockCustomList.locations } returns mockLocations
        every { mockCustomList.name } returns name
        every { mockLocation.countryCode } returns location.countryCode
        coEvery { mockCustomListsRepository.deleteCustomList(id = customListId) } returns
            Unit.right()
        coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns
            mockCustomList.right()

        // Act
        val result = customListActionUseCase.performAction(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `update locations action should return success with changed locations`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val newLocations = listOf(GeoLocationId.Country("EF"), GeoLocationId.Country("GH"))
        val oldLocations: ArrayList<GeoLocationId> =
            arrayListOf(GeoLocationId.Country("AB"), GeoLocationId.Country("CD"))
        val customListId = CustomListId("1")
        val customList = CustomList(id = customListId, name = name, locations = oldLocations)
        val action = CustomListAction.UpdateLocations(id = customListId, locations = newLocations)
        val expectedResult =
            Result.success(
                CustomListResult.LocationsChanged(
                    name = name,
                    undo = action.not(locations = oldLocations)
                )
            )
        coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns
            customList.right()

        coEvery {
            mockCustomListsRepository.updateCustomListLocations(customListId, newLocations)
        } returns Unit.right()

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
