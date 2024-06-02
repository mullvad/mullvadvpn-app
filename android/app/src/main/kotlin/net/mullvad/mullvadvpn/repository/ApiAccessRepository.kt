package net.mullvad.mullvadvpn.repository

import arrow.core.raise.either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.GetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.NewAccessMethod

class ApiAccessRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val accessMethods =
        managementService.settings
            .mapNotNull { it.apiAccessMethods }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun addApiAccessMethod(newAccessMethod: NewAccessMethod) =
        managementService.addApiAccessMethod(newAccessMethod)

    suspend fun removeApiAccessMethod(apiAccessMethodId: ApiAccessMethodId) =
        managementService.removeApiAccessMethod(apiAccessMethodId)

    suspend fun setApiAccessMethod(apiAccessMethodId: ApiAccessMethodId) =
        managementService.setApiAccessMethod(apiAccessMethodId)

    suspend fun updateApiAccessMethod(apiAccessMethod: ApiAccessMethod) =
        managementService.updateApiAccessMethod(apiAccessMethod)

    suspend fun getCurrentApiAccessMethod() = managementService.getCurrentApiAccessMethod()

    suspend fun testCustomApiAccessMethod(customProxy: ApiAccessMethodType.CustomProxy) =
        managementService.testCustomApiAccessMethod(customProxy)

    suspend fun testApiAccessMethodById(apiAccessMethodId: ApiAccessMethodId) =
        managementService.testApiAccessMethodById(apiAccessMethodId)

    suspend fun getApiAccessMethodById(id: ApiAccessMethodId) =
        either<GetApiAccessMethodError, ApiAccessMethod> {
            accessMethods.value?.firstOrNull { it.id == id }
                ?: raise(GetApiAccessMethodError.NotFound)
        }
}
