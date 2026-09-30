package net.mullvad.mullvadvpn.lib.repository

import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import java.util.Locale
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListAlreadyExists
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.Settings
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

@ExperimentalCoroutinesApi
class CustomListsRepositoryTest {
    private val mockManagementService: ManagementService = mockk()
    private val mockLocaleRepository: LocaleRepository = mockk()
    private lateinit var customListsRepository: CustomListsRepository

    private val settingsFlow: MutableStateFlow<Settings> = MutableStateFlow(mockk(relaxed = true))
    private val localeFlow: MutableStateFlow<Locale> = MutableStateFlow(mockk(relaxed = true))

    private lateinit var defaultLocale: Locale

    @BeforeEach
    fun setup() {
        defaultLocale = Locale.getDefault()
        every { mockManagementService.settings } returns settingsFlow
        every { mockLocaleRepository.currentLocale } returns localeFlow
        customListsRepository =
            CustomListsRepository(
                managementService = mockManagementService,
                localeRepository = mockLocaleRepository,
                dispatcher = UnconfinedTestDispatcher(),
            )
    }

    @AfterEach
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun getCustomListByIdShouldReturnCustomListWhenIdMatchesCustomListInSettings() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val mockCustomList =
            CustomList(
                id = customListId,
                name = mockk(relaxed = true),
                locations = listOf(mockk(relaxed = true)),
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
    fun getCustomListByIdShouldReturnGetCustomListErrorWhenIdDoesNotMatchesCustomListInSettings() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val mockCustomList =
                CustomList(
                    id = customListId,
                    name = mockk(relaxed = true),
                    locations = listOf(mockk(relaxed = true)),
                )
            val mockSettings: Settings = mockk()
            val otherCustomListId = CustomListId("2")
            every { mockSettings.customLists } returns listOf(mockCustomList)
            settingsFlow.value = mockSettings

            // Act
            val result = customListsRepository.getCustomListById(otherCustomListId)

            // Assert
            assertEquals(GetCustomListError(otherCustomListId), result.leftOrNull())
        }

    @Test
    fun createCustomListShouldReturnIdWhenCreationIsSuccessful() = runTest {
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
    fun createCustomListShouldReturnListsExistsErrorFromManagementService() = runTest {
        // Arrange
        val expectedResult = CustomListAlreadyExists.left()
        val customListName = CustomListName.fromString("CUSTOM")
        coEvery { mockManagementService.createCustomList(customListName) } returns expectedResult

        // Act
        val result = customListsRepository.createCustomList(customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun updateCustomListNameShouldReturnSuccessWhenCallManagementServiceIsSuccessful() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val expectedResult = Unit.right()
        val customListName = CustomListName.fromString("CUSTOM")
        val mockSettings: Settings = mockk()
        val mockCustomList =
            CustomList(
                id = customListId,
                name = mockk(relaxed = true),
                locations = listOf(mockk(relaxed = true)),
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
    fun updateCustomListNameShouldReturnListExistsErrorWhenListExistsErrorIsReceived() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val customListName = CustomListName.fromString("CUSTOM")
        val expectedResult = NameAlreadyExists(customListName).left()
        val mockSettings: Settings = mockk()
        val mockCustomList =
            CustomList(
                id = customListId,
                name = CustomListName.fromString("OLD CUSTOM"),
                locations = emptyList(),
            )
        val updatedCustomList =
            CustomList(id = customListId, name = customListName, locations = emptyList())
        every { mockSettings.customLists } returns listOf(mockCustomList)
        settingsFlow.value = mockSettings
        coEvery { mockManagementService.updateCustomList(updatedCustomList) } returns expectedResult

        // Act
        val result = customListsRepository.updateCustomListName(customListId, customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun whenDeleteCustomListsIsCalledManagementserviceDeleteCustomListShouldBeCalled() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        coEvery { mockManagementService.deleteCustomList(customListId) } returns Unit.right()

        // Act
        customListsRepository.deleteCustomList(customListId)

        // Assert
        coVerify { mockManagementService.deleteCustomList(customListId) }
    }

    @Test
    fun updateCustomListLocationsShouldReturnSuccessfulWhenListExistsAndUpdateIsSuccessful() =
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
    fun updateCustomListLocationsShouldReturnGetCustomListErrorWhenListDoesNotExist() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val customListId = CustomListId("1")
        val otherCustomListId = CustomListId("2")
        val expectedResult = GetCustomListError(otherCustomListId).left()
        val mockCustomList =
            CustomList(
                id = customListId,
                name = CustomListName.fromString("name"),
                locations = emptyList(),
            )
        val locationId = GeoLocationId.Country("se")
        every { mockSettings.customLists } returns listOf(mockCustomList)
        settingsFlow.value = mockSettings

        // Act
        val result =
            customListsRepository.updateCustomListLocations(
                otherCustomListId,
                listOf(locationId),
            )

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun customListsShouldBeSortedAlphabeticallyByName() = runTest {
        // Arrange
        val customListId1 = CustomListId("1")
        val customListId2 = CustomListId("2")
        val customListId3 = CustomListId("3")
        val customList1 =
            CustomList(
                id = customListId1,
                name = CustomListName.fromString("Z List"),
                locations = emptyList(),
            )
        val customList2 =
            CustomList(
                id = customListId2,
                name = CustomListName.fromString("A List"),
                locations = emptyList(),
            )
        val customList3 =
            CustomList(
                id = customListId3,
                name = CustomListName.fromString("M List"),
                locations = emptyList(),
            )
        val mockSettings: Settings = mockk()
        every { mockSettings.customLists } returns listOf(customList1, customList2, customList3)
        settingsFlow.value = mockSettings

        // Act
        val result = customListsRepository.customLists.value

        // Assert
        val expectedOrder = listOf(customList2, customList3, customList1)
        assertEquals(expectedOrder, result)
    }

    @Test
    fun customListsShouldBeSortedUsingPtCollation() = runTest {
        // Arrange
        Locale.setDefault(Locale.forLanguageTag("pt"))
        val customListId1 = CustomListId("1")
        val customListId2 = CustomListId("2")
        val customListId3 = CustomListId("3")
        val customList1 =
            CustomList(
                id = customListId1,
                name = CustomListName.fromString("Z List"),
                locations = emptyList(),
            )
        val customList2 =
            CustomList(
                id = customListId2,
                name = CustomListName.fromString("Á List"),
                locations = emptyList(),
            )
        val customList3 =
            CustomList(
                id = customListId3,
                name = CustomListName.fromString("B List"),
                locations = emptyList(),
            )
        val mockSettings: Settings = mockk()
        every { mockSettings.customLists } returns listOf(customList1, customList2, customList3)
        settingsFlow.value = mockSettings

        // Act
        val result = customListsRepository.customLists.value

        // Assert
        // 'Á' (U+00C1) outranks 'Z' (U+005A) by Unicode value, so comparing names by Unicode
        // value would place "Ávila" after "Zurich".
        val expectedOrder = listOf(customList2, customList3, customList1)
        assertEquals(expectedOrder, result)
    }
}
