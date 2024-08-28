package net.mullvad.mullvadvpn.repository

import arrow.core.raise.either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.GetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting

class ApiAccessRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val accessMethods =
        managementService.settings
            .mapNotNull { it.apiAccessMethodSettings }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    val currentAccessMethod =
        managementService.currentAccessMethod.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.Eagerly,
            null,
        )

    suspend fun addApiAccessMethod(newAccessMethodSetting: NewAccessMethodSetting) =
        managementService.addApiAccessMethod(newAccessMethodSetting)

    suspend fun removeApiAccessMethod(apiAccessMethodId: ApiAccessMethodId) =
        managementService.removeApiAccessMethod(apiAccessMethodId)

    suspend fun setCurrentApiAccessMethod(apiAccessMethodId: ApiAccessMethodId) =
        managementService.setApiAccessMethod(apiAccessMethodId)

    private suspend fun updateApiAccessMethod(apiAccessMethodSetting: ApiAccessMethodSetting) =
        managementService.updateApiAccessMethod(apiAccessMethodSetting)

    suspend fun updateApiAccessMethod(
        apiAccessMethodId: ApiAccessMethodId,
        apiAccessMethodName: ApiAccessMethodName,
        apiAccessMethod: ApiAccessMethod,
    ) = either {
        val apiAccessMethodSetting = getApiAccessMethodSettingById(apiAccessMethodId).bind()
        updateApiAccessMethod(
                apiAccessMethodSetting.copy(
                    id = apiAccessMethodId,
                    name = apiAccessMethodName,
                    apiAccessMethod = apiAccessMethod,
                )
            )
            .bind()
    }

    suspend fun testCustomApiAccessMethod(customProxy: ApiAccessMethod.CustomProxy) =
        managementService.testCustomApiAccessMethod(customProxy)

    suspend fun testApiAccessMethodById(apiAccessMethodId: ApiAccessMethodId) =
        managementService.testApiAccessMethodById(apiAccessMethodId)

    fun getApiAccessMethodSettingById(id: ApiAccessMethodId) =
        either<GetApiAccessMethodError, ApiAccessMethodSetting> {
            accessMethods.value?.firstOrNull { it.id == id }
                ?: raise(GetApiAccessMethodError.NotFound)
        }

    fun apiAccessMethodSettingById(id: ApiAccessMethodId): Flow<ApiAccessMethodSetting> =
        accessMethods.mapNotNull { it?.firstOrNull { accessMethod -> accessMethod.id == id } }

    fun enabledApiAccessMethods(): Flow<List<ApiAccessMethodSetting>> =
        accessMethods.mapNotNull { it?.filter { accessMethod -> accessMethod.enabled } }

    suspend fun setEnabledApiAccessMethod(id: ApiAccessMethodId, enabled: Boolean) = either {
        val accessMethod = getApiAccessMethodSettingById(id).bind()
        updateApiAccessMethod(accessMethod.copy(enabled = enabled)).bind()
    }
}
