package net.mullvad.mullvadvpn.repository

import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class CustomListsRepositoryTest {
    private val mockManagementService: ManagementService = mockk()
    private lateinit var customListsRepository: CustomListsRepository

    private val settingsFlow: MutableStateFlow<Settings> = MutableStateFlow(mockk(relaxed = true))

    @BeforeEach
    fun setup() {
        mockkStatic(RELAY_LIST_EXTENSIONS)
        every { mockManagementService.settings } returns settingsFlow
        customListsRepository =
            CustomListsRepository(
                managementService = mockManagementService,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @Test
    fun `get custom list by id should return custom list when id matches custom list in settings`() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val mockCustomList =
                CustomList(
                    id = customListId,
                    name = mockk(relaxed = true),
                    locations = mockk(relaxed = true)
                )
            val mockSettings: Settings = mockk()
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings

            // Act
            val result = customListsRepository.getCustomListById(customListId)

            // Assert
            assertEquals(mockCustomList, result.getOrNull())
        }

    @Test
    fun `get custom list by id should return get custom list error when id does not matches custom list in settings`() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val mockCustomList =
                CustomList(
                    id = customListId,
                    name = mockk(relaxed = true),
                    locations = mockk(relaxed = true)
                )
            val mockSettings: Settings = mockk()
            val otherCustomListId = CustomListId("2")
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings

            // Act
            val result = customListsRepository.getCustomListById(otherCustomListId)

            // Assert
            assertEquals(GetCustomListError, result.leftOrNull())
        }

    @Test
    fun `create custom list should return id when creation is successful`() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val expectedResult = customListId.right()
        val customListName = CustomListName.fromString("CUSTOM")
        coEvery { mockManagementService.createCustomList(customListName) } returns expectedResult

        // Act
        val result = customListsRepository.createCustomList(customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `create custom list should return lists exists when lists exists error event is received`() =
        runTest {
            // Arrange
            val expectedResult = CreateCustomListError.CustomListAlreadyExists.left()
            val customListName = CustomListName.fromString("CUSTOM")
            coEvery { mockManagementService.createCustomList(customListName) } returns
                expectedResult

            // Act
            val result = customListsRepository.createCustomList(customListName)

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `update custom list name should return either right unit when successful`() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val expectedResult = Unit.right()
        val customListName = CustomListName.fromString("CUSTOM")
        val mockSettings: Settings = mockk()
        val mockCustomList =
            CustomList(
                id = customListId,
                name = mockk(relaxed = true),
                locations = mockk(relaxed = true)
            )
        every { mockSettings.customLists } returns listOf(mockCustomList)
        settingsFlow.value = mockSettings
        coEvery { mockManagementService.updateCustomList(any<CustomList>()) } returns expectedResult

        // Act
        val result = customListsRepository.updateCustomListName(customListId, customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `update custom list name should return list exists error when list exists error is received`() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val customListName = CustomListName.fromString("CUSTOM")
            val expectedResult =
                UpdateCustomListError.NameAlreadyExists(customListName.value).left()
            val mockSettings: Settings = mockk()
            val mockCustomList =
                CustomList(
                    id = customListId,
                    name = CustomListName.fromString("OLD CUSTOM"),
                    locations = emptyList()
                )
            val updatedCustomList =
                CustomList(id = customListId, name = customListName, locations = emptyList())
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings
            coEvery { mockManagementService.updateCustomList(updatedCustomList) } returns
                expectedResult

            // Act
            val result = customListsRepository.updateCustomListName(customListId, customListName)

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `when delete custom lists is called a delete custom event should be sent`() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        coEvery { mockManagementService.deleteCustomList(customListId) } returns Unit.right()

        // Act
        customListsRepository.deleteCustomList(customListId)

        // Assert
        coVerify { mockManagementService.deleteCustomList(customListId) }
    }

    @Test
    fun `update custom list locations should return Either right when list exists and update is successful`() =
        runTest {
            // Arrange
            val expectedResult = Unit.right()
            val customListId = CustomListId("1")
            val customListName = CustomListName.fromString("CUSTOM")
            val location = GeoLocationId.Country("se")
            val mockSettings: Settings = mockk()
            val mockCustomList =
                CustomList(id = customListId, name = customListName, locations = emptyList())
            val updatedCustomList =
                CustomList(id = customListId, name = customListName, locations = listOf(location))
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings
            coEvery { mockManagementService.updateCustomList(updatedCustomList) } returns
                Unit.right()

            // Act
            val result =
                customListsRepository.updateCustomListLocations(customListId, listOf(location))

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `update custom list locations should return get custom list error when list does not exist`() =
        runTest {
            // Arrange
            val expectedResult = GetCustomListError.left()
            val mockSettings: Settings = mockk()
            val customListId = CustomListId("1")
            val otherCustomListId = CustomListId("2")
            val mockCustomList =
                CustomList(
                    id = customListId,
                    name = CustomListName.fromString("name"),
                    locations = emptyList()
                )
            val locationId = GeoLocationId.Country("se")
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings

            // Act
            val result =
                customListsRepository.updateCustomListLocations(
                    otherCustomListId,
                    listOf(locationId)
                )

            // Assert
            assertEquals(expectedResult, result)
        }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
