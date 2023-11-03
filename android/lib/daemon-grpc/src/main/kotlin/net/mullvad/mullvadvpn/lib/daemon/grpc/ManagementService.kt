package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.LocalSocketAddress
import android.util.Log
import com.google.protobuf.Empty
import com.google.protobuf.StringValue
import io.grpc.Status
import io.grpc.StatusException
import io.grpc.android.UdsChannelBuilder
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import mullvad_daemon.management_interface.ManagementInterface.*
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory as ModelAccountHistory
import net.mullvad.mullvadvpn.model.AccountState
import net.mullvad.mullvadvpn.model.Device as ModelDevice
import net.mullvad.mullvadvpn.model.LoginResult
import org.joda.time.Instant

class ManagementService(
    rpcSocketPath: String,
    private val scope: CoroutineScope,
) {

    data class ManagementServiceState(
        val tunnelState: TunnelState? = null,
        val settings: Settings? = null,
        val relayList: RelayList? = null,
        val versionInfo: AppVersionInfo? = null,
        val device: DeviceState? = null,
        val deviceEvent: DeviceEvent? = null,
    )

    private val channel =
        UdsChannelBuilder.forPath(rpcSocketPath, LocalSocketAddress.Namespace.FILESYSTEM).build()
    private val managementService = ManagementServiceGrpcKt.ManagementServiceCoroutineStub(channel)

    private val _mutableStateFlow: MutableStateFlow<ManagementServiceState> =
        MutableStateFlow(ManagementServiceState())
    val state: StateFlow<ManagementServiceState> = _mutableStateFlow

    val deviceState: Flow<AccountState> =
        _mutableStateFlow
            .mapNotNull { it.device }
            .map {
                when (it.state) {
                    DeviceState.State.LOGGED_IN ->
                        AccountState.LoggedIn(
                            device =
                                ModelDevice(
                                    it.device.device.id,
                                    it.device.device.name,
                                    it.device.device.pubkey.toByteArray(),
                                    it.device.device.created.toString(),
                                ),
                            accountToken = it.device.accountToken
                        )
                    DeviceState.State.LOGGED_OUT -> AccountState.LoggedOut
                    DeviceState.State.REVOKED -> AccountState.Revoked
                    DeviceState.State.UNRECOGNIZED -> AccountState.Unrecognized
                }
            }

    init {
        // TODO This should be fixed
        scope.launch {
            delay(1000)
            start()
        }
    }

    suspend fun start() {
        scope.launch { _mutableStateFlow.update { getInitialServiceState() } }
        scope.launch {
            managementService.eventsListen(Empty.getDefaultInstance()).collect { event ->
                @Suppress("WHEN_ENUM_CAN_BE_NULL_IN_JAVA")
                when (event.eventCase) {
                    DaemonEvent.EventCase.TUNNEL_STATE ->
                        _mutableStateFlow.update { it.copy(tunnelState = event.tunnelState) }
                    DaemonEvent.EventCase.SETTINGS ->
                        _mutableStateFlow.update { it.copy(settings = event.settings) }
                    DaemonEvent.EventCase.RELAY_LIST ->
                        _mutableStateFlow.update { it.copy(relayList = event.relayList) }
                    DaemonEvent.EventCase.VERSION_INFO ->
                        _mutableStateFlow.update { it.copy(versionInfo = event.versionInfo) }
                    DaemonEvent.EventCase.DEVICE ->
                        _mutableStateFlow.update { it.copy(device = event.device.newState) }
                    DaemonEvent.EventCase.REMOVE_DEVICE -> {}
                    DaemonEvent.EventCase.EVENT_NOT_SET -> {}
                }
            }
        }
    }

    suspend fun getDevice(): DeviceState = managementService.getDevice(Empty.getDefaultInstance())

    suspend fun getTunnelState(): TunnelState =
        managementService.getTunnelState(Empty.getDefaultInstance())

    suspend fun getSettings(): Settings = managementService.getSettings(Empty.getDefaultInstance())

    suspend fun getRelayList(): RelayList =
        managementService.getRelayLocations(Empty.getDefaultInstance())

    suspend fun getVersionInfo(): AppVersionInfo =
        managementService.getVersionInfo(Empty.getDefaultInstance())

    suspend fun logoutAccount(): Unit {
        managementService.logoutAccount(Empty.getDefaultInstance())
    }

    suspend fun loginAccount(accountToken: String): LoginResult {
        return try {
            managementService.loginAccount(StringValue.of(accountToken))
            LoginResult.Ok
        } catch (e: StatusException) {
            when (e.status.code) {
                Status.Code.OK -> TODO()
                Status.Code.RESOURCE_EXHAUSTED -> LoginResult.MaxDevicesReached
                Status.Code.UNAVAILABLE -> LoginResult.RpcError
                Status.Code.UNAUTHENTICATED -> LoginResult.InvalidAccount
                Status.Code.CANCELLED -> TODO()
                Status.Code.UNKNOWN -> TODO()
                Status.Code.INVALID_ARGUMENT -> TODO()
                Status.Code.DEADLINE_EXCEEDED -> TODO()
                Status.Code.NOT_FOUND -> TODO()
                Status.Code.ALREADY_EXISTS -> TODO()
                Status.Code.PERMISSION_DENIED -> TODO()
                Status.Code.FAILED_PRECONDITION -> TODO()
                Status.Code.ABORTED -> TODO()
                Status.Code.OUT_OF_RANGE -> TODO()
                Status.Code.UNIMPLEMENTED -> TODO()
                Status.Code.INTERNAL -> TODO()
                Status.Code.DATA_LOSS -> TODO()
            }
        }
    }

    suspend fun clearAccountHistory(): Unit {
        managementService.clearAccountHistory(Empty.getDefaultInstance())
    }

    suspend fun getAccountHistory() =
        try {
            val history = managementService.getAccountHistory(Empty.getDefaultInstance())
            if (history.hasToken()) {
                ModelAccountHistory.Available(history.token.value)
            } else {
                ModelAccountHistory.Missing
            }
        } catch (e: StatusException) {
            throw e
        }

    private suspend fun getInitialServiceState() =
        ManagementServiceState(
            getTunnelState(),
            getSettings(),
            getRelayList(),
            getVersionInfo(),
            getDevice(),
        )

    suspend fun getAccountExpiry(accountToken: String): AccountExpiry =
        try {
            val expiry = managementService.getAccountData(StringValue.of(accountToken))
            if (expiry.hasExpiry()) {
                AccountExpiry.Available(Instant.ofEpochSecond(expiry.expiry.seconds).toDateTime())
            } else {
                AccountExpiry.Missing
            }
        } catch (e: StatusException) {
            throw e
        }

    suspend fun createAccount(): AccountCreationResult =
        try {
            val accountTokenStringValue =
                managementService.createNewAccount(Empty.getDefaultInstance())
            AccountCreationResult.Success(accountTokenStringValue.value)
        } catch (e: StatusException) {
            Log.e("ManagementService", "createAccount error: ${e.message}")
            AccountCreationResult.Failure
        }
}
