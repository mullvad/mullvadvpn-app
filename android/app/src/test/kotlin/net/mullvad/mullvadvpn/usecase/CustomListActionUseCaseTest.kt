package net.mullvad.mullvadvpn.usecase

import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListAlreadyExists
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CreateWithLocationsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.RenameError
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class CustomListActionUseCaseTest {
    private val mockCustomListsRepository: CustomListsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val customListActionUseCase =
        CustomListActionUseCase(
            customListsRepository = mockCustomListsRepository,
            relayListRepository = mockRelayListRepository,
        )

    private val relayListFlow = MutableStateFlow(emptyList<RelayItem.Location.Country>())

    @BeforeEach
    fun setup() {
        mockkStatic(RELAY_LIST_EXTENSIONS)
        every { mockRelayListRepository.relayList } returns relayListFlow
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
            Created(
                    id = createdId,
                    name = name,
                    locationNames = listOf(locationName),
                    undo = action.not(createdId),
                )
                .right()
        coEvery { mockCustomListsRepository.createCustomList(name) } returns createdId.right()
        coEvery {
            mockCustomListsRepository.updateCustomList(
                CustomList(createdId, name, listOf(locationId))
            )
        } returns Unit.right()
        relayListFlow.value =
            listOf(
                RelayItem.Location.Country(
                    id = locationId,
                    name = locationName,
                    cities = emptyList(),
                )
            )

        // Act
        val result = customListActionUseCase(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `create action should return error when name already exists`() = runTest {
        // Arrange
        val name = CustomListName.fromString("test")
        val locationId = GeoLocationId.Country("AB")
        val action = CustomListAction.Create(name = name, locations = listOf(locationId))
        val expectedError = CreateWithLocationsError.Create(CustomListAlreadyExists).left()
        coEvery { mockCustomListsRepository.createCustomList(name) } returns
            CustomListAlreadyExists.left()

        // Act
        val result = customListActionUseCase(action)

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
        val expectedResult = Renamed(undo = action.not()).right()
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns Unit.right()

        // Act
        val result = customListActionUseCase(action)

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
        coEvery {
            mockCustomListsRepository.updateCustomListName(id = customListId, name = newName)
        } returns NameAlreadyExists(newName).left()

        val expectedError = RenameError(NameAlreadyExists(newName)).left()

        // Act
        val result = customListActionUseCase(action)

        // Assert
        assertEquals(expectedError, result)
    }

    @Test
    fun `delete action should return successful with deleted list`() = runTest {
        // Arrange
        val mockLocation: GeoLocationId.Country = mockk()
        val mockLocations: List<GeoLocationId> = listOf(mockLocation)
        val name = CustomListName.fromString("test")
        val customListId = CustomListId("1")
        val mockCustomList = CustomList(id = customListId, name = name, locations = mockLocations)
        val location = GeoLocationId.Country("AB")
        val action = CustomListAction.Delete(id = customListId)
        val expectedResult =
            Deleted(undo = action.not(name = name, locations = listOf(location))).right()
        every { mockLocation.code } returns location.code
        coEvery { mockCustomListsRepository.deleteCustomList(id = customListId) } returns
            Unit.right()
        coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns
            mockCustomList.right()

        // Act
        val result = customListActionUseCase(action)

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
            LocationsChanged(
                    id = customListId,
                    name = name,
                    locations = newLocations,
                    oldLocations = oldLocations,
                )
                .right()
        coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns
            customList.right()

        coEvery {
            mockCustomListsRepository.updateCustomListLocations(customListId, newLocations)
        } returns Unit.right()

        // Act
        val result = customListActionUseCase(action)

        // Assert
        assertEquals(expectedResult, result)
    }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
