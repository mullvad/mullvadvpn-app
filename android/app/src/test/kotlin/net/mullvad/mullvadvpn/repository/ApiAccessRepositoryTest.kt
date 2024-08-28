package net.mullvad.mullvadvpn.repository

import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AddApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.GetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.SetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.UnknownApiAccessMethodError
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ApiAccessRepositoryTest {
    private val mockManagementService: ManagementService = mockk()

    private lateinit var apiAccessRepository: ApiAccessRepository

    private val settingsFlow: MutableStateFlow<Settings> = MutableStateFlow(mockk(relaxed = true))

    @BeforeEach
    fun setUp() {
        every { mockManagementService.settings } returns settingsFlow
        every { mockManagementService.currentAccessMethod } returns emptyFlow()

        apiAccessRepository =
            ApiAccessRepository(
                managementService = mockManagementService,
                dispatcher = UnconfinedTestDispatcher(),
            )
    }

    @Test
    fun `adding api access method should return id when successful`() = runTest {
        // Arrange
        val newAccessMethodSetting: NewAccessMethodSetting = mockk()
        val accessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        coEvery { mockManagementService.addApiAccessMethod(newAccessMethodSetting) } returns
            accessMethodId.right()

        // Act
        val result = apiAccessRepository.addApiAccessMethod(newAccessMethodSetting)

        // Assert
        coVerify { mockManagementService.addApiAccessMethod(newAccessMethodSetting) }
        assertEquals(accessMethodId.right(), result)
    }

    @Test
    fun `adding api access method should return error when not successful`() = runTest {
        // Arrange
        val newAccessMethodSetting: NewAccessMethodSetting = mockk()
        val addApiAccessMethodError: AddApiAccessMethodError.Unknown = mockk()
        coEvery { mockManagementService.addApiAccessMethod(newAccessMethodSetting) } returns
            addApiAccessMethodError.left()

        // Act
        val result = apiAccessRepository.addApiAccessMethod(newAccessMethodSetting)

        // Assert
        coVerify { mockManagementService.addApiAccessMethod(newAccessMethodSetting) }
        assertEquals(addApiAccessMethodError.left(), result)
    }

    @Test
    fun `setting api access method should return successful when successful`() = runTest {
        // Arrange
        val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        coEvery { mockManagementService.setApiAccessMethod(apiAccessMethodId) } returns Unit.right()

        // Act
        val result = apiAccessRepository.setCurrentApiAccessMethod(apiAccessMethodId)

        // Assert
        coVerify { mockManagementService.setApiAccessMethod(apiAccessMethodId) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `setting api access method should return error when not successful`() = runTest {
        // Arrange
        val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        val setApiAccessMethodError: SetApiAccessMethodError = mockk()
        coEvery { mockManagementService.setApiAccessMethod(apiAccessMethodId) } returns
            setApiAccessMethodError.left()

        // Act
        val result = apiAccessRepository.setCurrentApiAccessMethod(apiAccessMethodId)

        // Assert
        coVerify { mockManagementService.setApiAccessMethod(apiAccessMethodId) }
        assertEquals(setApiAccessMethodError.left(), result)
    }

    @Test
    fun `test api access method by id should return successful when successful`() = runTest {
        // Arrange
        val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        coEvery { mockManagementService.testApiAccessMethodById(apiAccessMethodId) } returns
            Unit.right()

        // Act
        val result = apiAccessRepository.testApiAccessMethodById(apiAccessMethodId)

        // Assert
        coVerify { mockManagementService.testApiAccessMethodById(apiAccessMethodId) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `test api access method by id should return error when not successful`() = runTest {
        // Arrange
        val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
        val testApiAccessMethodError: TestApiAccessMethodError = mockk()
        coEvery { mockManagementService.testApiAccessMethodById(apiAccessMethodId) } returns
            testApiAccessMethodError.left()

        // Act
        val result = apiAccessRepository.testApiAccessMethodById(apiAccessMethodId)

        // Assert
        coVerify { mockManagementService.testApiAccessMethodById(apiAccessMethodId) }
        assertEquals(testApiAccessMethodError.left(), result)
    }

    @Test
    fun `test custom api access method should return successful when successful`() = runTest {
        // Arrange
        val customProxy: ApiAccessMethod.CustomProxy = mockk()
        coEvery { mockManagementService.testCustomApiAccessMethod(customProxy) } returns
            Unit.right()

        // Act
        val result = apiAccessRepository.testCustomApiAccessMethod(customProxy)

        // Assert
        coVerify { mockManagementService.testCustomApiAccessMethod(customProxy) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `test custom api access method should return error when not successful`() = runTest {
        // Arrange
        val customProxy: ApiAccessMethod.CustomProxy = mockk()
        val testApiAccessMethodError: TestApiAccessMethodError = mockk()
        coEvery { mockManagementService.testCustomApiAccessMethod(customProxy) } returns
            testApiAccessMethodError.left()

        // Act
        val result = apiAccessRepository.testCustomApiAccessMethod(customProxy)

        // Assert
        coVerify { mockManagementService.testCustomApiAccessMethod(customProxy) }
        assertEquals(testApiAccessMethodError.left(), result)
    }

    @Test
    fun `get access method by id should return access method when id matches in settings`() =
        runTest {
            // Arrange
            val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val expectedResult =
                ApiAccessMethodSetting(
                    name = ApiAccessMethodName.fromString("Name"),
                    apiAccessMethod = ApiAccessMethod.Direct,
                    enabled = true,
                    id = apiAccessMethodId,
                )
            val mockSettings: Settings = mockk()
            every { mockSettings.apiAccessMethodSettings } returns listOf(expectedResult)
            settingsFlow.value = mockSettings

            // Act
            val result = apiAccessRepository.getApiAccessMethodSettingById(apiAccessMethodId)

            // Assert
            assertEquals(expectedResult.right(), result)
        }

    @Test
    fun `get access method by id should return not found error when id does not matches in settings`() =
        runTest {
            // Arrange
            val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val expectedError = GetApiAccessMethodError.NotFound
            val mockSettings: Settings = mockk()
            every { mockSettings.apiAccessMethodSettings } returns emptyList()
            settingsFlow.value = mockSettings

            // Act
            val result = apiAccessRepository.getApiAccessMethodSettingById(apiAccessMethodId)

            // Assert
            assertEquals(expectedError.left(), result)
        }

    @Test
    fun `when setting enable for api access method should return successful when successful`() =
        runTest {
            // Arrange
            val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val apiAccessMethodSetting =
                ApiAccessMethodSetting(
                    name = ApiAccessMethodName.fromString("Name"),
                    apiAccessMethod = ApiAccessMethod.Direct,
                    enabled = true,
                    id = apiAccessMethodId,
                )
            val mockSettings: Settings = mockk()
            every { mockSettings.apiAccessMethodSettings } returns listOf(apiAccessMethodSetting)
            coEvery { mockManagementService.updateApiAccessMethod(apiAccessMethodSetting) } returns
                Unit.right()
            settingsFlow.value = mockSettings

            // Act
            val result = apiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, true)

            // Assert
            assertEquals(Unit.right(), result)
        }

    @Test
    fun `when setting enable for api access method should return error when not method not found`() =
        runTest {
            // Arrange
            val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val expectedError = GetApiAccessMethodError.NotFound
            val mockSettings: Settings = mockk()
            every { mockSettings.apiAccessMethodSettings } returns emptyList()
            settingsFlow.value = mockSettings

            // Act
            val result = apiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, true)

            // Assert
            assertEquals(expectedError.left(), result)
        }

    @Test
    fun `when setting enable for api access method should return error when not successful`() =
        runTest {
            // Arrange
            val expectedError: UnknownApiAccessMethodError = mockk()
            val apiAccessMethodId: ApiAccessMethodId = ApiAccessMethodId.fromString(UUID)
            val apiAccessMethodSetting =
                ApiAccessMethodSetting(
                    name = ApiAccessMethodName.fromString("Name"),
                    apiAccessMethod = ApiAccessMethod.Direct,
                    enabled = true,
                    id = apiAccessMethodId,
                )
            val mockSettings: Settings = mockk()
            every { mockSettings.apiAccessMethodSettings } returns listOf(apiAccessMethodSetting)
            coEvery { mockManagementService.updateApiAccessMethod(apiAccessMethodSetting) } returns
                expectedError.left()
            settingsFlow.value = mockSettings

            // Act
            val result = apiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, true)

            // Assert
            assertEquals(expectedError.left(), result)
        }
}
