package net.mullvad.mullvadvpn.repository

import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.getGeographicLocationConstraintByCode
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class CustomListsRepositoryTest {
    private val mockMessageHandler: MessageHandler = mockk()
    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockRelayListListener: RelayListListener = mockk()
    private val customListsRepository =
        CustomListsRepository(
            messageHandler = mockMessageHandler,
            settingsRepository = mockSettingsRepository,
            relayListListener = mockRelayListListener
        )

    private val settingsFlow: MutableStateFlow<Settings?> = MutableStateFlow(null)
    private val relayListFlow: MutableStateFlow<RelayList> = MutableStateFlow(mockk())

    @BeforeEach
    fun setup() {
        mockkStatic(RELAY_LIST_EXTENSIONS)
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow
        every { mockRelayListListener.relayListEvents } returns relayListFlow
    }

    @Test
    fun `get custom list by id should return custom list when id matches custom list in settings`() {
        // Arrange
        val mockCustomList: CustomList = mockk()
        val mockSettings: Settings = mockk()
        val customListId = "1"
        settingsFlow.value = mockSettings
        every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)
        every { mockCustomList.id } returns customListId

        // Act
        val result = customListsRepository.getCustomListById(customListId)

        // Assert
        assertEquals(mockCustomList, result)
    }

    @Test
    fun `get custom list by id should return null when id does not matches custom list in settings`() {
        // Arrange
        val mockCustomList: CustomList = mockk()
        val mockSettings: Settings = mockk()
        val customListId = "1"
        val otherCustomListId = "2"
        settingsFlow.value = mockSettings
        every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)
        every { mockCustomList.id } returns customListId

        // Act
        val result = customListsRepository.getCustomListById(otherCustomListId)

        // Assert
        assertNull(result)
    }

    @Test
    fun `create custom list should return Ok when creation is successful`() = runTest {
        // Arrange
        val customListId = "1"
        val expectedResult = CreateCustomListResult.Ok(customListId)
        val customListName = "CUSTOM"
        every {
            mockMessageHandler.trySendRequest(Request.CreateCustomList(customListName))
        } returns true
        every { mockMessageHandler.events<Event.CreateCustomListResultEvent>() } returns
            flowOf(Event.CreateCustomListResultEvent(expectedResult))

        // Act
        val result = customListsRepository.createCustomList(customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `create custom list should return lists exists when lists exists error event is received`() =
        runTest {
            // Arrange
            val expectedResult = CreateCustomListResult.Error(CustomListsError.CustomListExists)
            val customListName = "CUSTOM"
            every {
                mockMessageHandler.trySendRequest(Request.CreateCustomList(customListName))
            } returns true
            every { mockMessageHandler.events<Event.CreateCustomListResultEvent>() } returns
                flowOf(Event.CreateCustomListResultEvent(expectedResult))

            // Act
            val result = customListsRepository.createCustomList(customListName)

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `update custom list name should return ok when list updated event is received`() = runTest {
        // Arrange
        val customListId = "1"
        val expectedResult = UpdateCustomListResult.Ok
        val customListName = "CUSTOM"
        val mockSettings: Settings = mockk()
        val mockCustomList: CustomList = mockk()
        val updatedCustomList: CustomList = mockk()
        settingsFlow.value = mockSettings
        every { mockCustomList.id } returns customListId
        every { mockCustomList.copy(customListId, customListName, any()) } returns updatedCustomList
        every {
            mockMessageHandler.trySendRequest(Request.UpdateCustomList(updatedCustomList))
        } returns true
        every { mockMessageHandler.events<Event.UpdateCustomListResultEvent>() } returns
            flowOf(Event.UpdateCustomListResultEvent(expectedResult))
        every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)

        // Act
        val result = customListsRepository.updateCustomListName(customListId, customListName)

        // Assert
        assertEquals(expectedResult, result)
    }

    @Test
    fun `update custom list name should return list exists error when list exists error is received`() =
        runTest {
            // Arrange
            val customListId = "1"
            val expectedResult = UpdateCustomListResult.Error(CustomListsError.CustomListExists)
            val customListName = "CUSTOM"
            val mockSettings: Settings = mockk()
            val mockCustomList: CustomList = mockk()
            val updatedCustomList: CustomList = mockk()
            settingsFlow.value = mockSettings
            every { mockCustomList.id } returns customListId
            every { mockCustomList.copy(customListId, customListName, any()) } returns
                updatedCustomList
            every {
                mockMessageHandler.trySendRequest(Request.UpdateCustomList(updatedCustomList))
            } returns true
            every { mockMessageHandler.events<Event.UpdateCustomListResultEvent>() } returns
                flowOf(Event.UpdateCustomListResultEvent(expectedResult))
            every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)

            // Act
            val result = customListsRepository.updateCustomListName(customListId, customListName)

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `when delete custom lists is called a delete custom event should be sent`() = runTest {
        // Arrange
        val customListId = "1"
        every { mockMessageHandler.trySendRequest(Request.DeleteCustomList(customListId)) } returns
            true

        // Act
        customListsRepository.deleteCustomList(customListId)

        // Assert
        verify { mockMessageHandler.trySendRequest(Request.DeleteCustomList(customListId)) }
    }

    @Test
    fun `update custom list locations should return ok when list exists and ok updated list event is received`() =
        runTest {
            // Arrange
            val expectedResult = UpdateCustomListResult.Ok
            val customListId = "1"
            val customListName = "CUSTOM"
            val locationCode = "AB"
            val mockSettings: Settings = mockk()
            val mockRelayList: RelayList = mockk()
            val mockCustomList: CustomList = mockk()
            val updatedCustomList: CustomList = mockk()
            val mockLocationConstraint: GeographicLocationConstraint = mockk()
            settingsFlow.value = mockSettings
            relayListFlow.value = mockRelayList
            every { mockCustomList.id } returns customListId
            every { mockCustomList.name } returns customListName
            every {
                mockCustomList.copy(
                    customListId,
                    customListName,
                    arrayListOf(mockLocationConstraint)
                )
            } returns updatedCustomList
            every {
                mockMessageHandler.trySendRequest(Request.UpdateCustomList(updatedCustomList))
            } returns true
            every { mockMessageHandler.events<Event.UpdateCustomListResultEvent>() } returns
                flowOf(Event.UpdateCustomListResultEvent(expectedResult))
            every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)
            every { mockRelayList.getGeographicLocationConstraintByCode(locationCode) } returns
                mockLocationConstraint

            // Act
            val result =
                customListsRepository.updateCustomListLocationsFromCodes(
                    customListId,
                    listOf(locationCode)
                )

            // Assert
            assertEquals(expectedResult, result)
        }

    @Test
    fun `update custom list locations should return other error when list does not exist`() =
        runTest {
            // Arrange
            val expectedResult = UpdateCustomListResult.Error(CustomListsError.OtherError)
            val mockCustomList: CustomList = mockk()
            val mockSettings: Settings = mockk()
            val customListId = "1"
            val otherCustomListId = "2"
            val locationCode = "AB"
            val mockRelayList: RelayList = mockk()
            val mockLocationConstraint: GeographicLocationConstraint = mockk()
            settingsFlow.value = mockSettings
            relayListFlow.value = mockRelayList
            every { mockSettings.customLists.customLists } returns arrayListOf(mockCustomList)
            every { mockCustomList.id } returns customListId
            every { mockRelayList.getGeographicLocationConstraintByCode(locationCode) } returns
                mockLocationConstraint

            // Act
            val result =
                customListsRepository.updateCustomListLocationsFromCodes(
                    otherCustomListId,
                    listOf(locationCode)
                )

            // Assert
            assertEquals(expectedResult, result)
        }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
