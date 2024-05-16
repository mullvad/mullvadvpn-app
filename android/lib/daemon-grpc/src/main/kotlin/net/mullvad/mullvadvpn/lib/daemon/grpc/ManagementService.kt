package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.LocalSocketAddress
import android.util.Log
import arrow.core.Either
import arrow.optics.copy
import arrow.optics.dsl.index
import arrow.optics.typeclasses.Index
import com.google.protobuf.BoolValue
import com.google.protobuf.Empty
import com.google.protobuf.StringValue
import com.google.protobuf.UInt32Value
import io.grpc.ConnectivityState
import io.grpc.Status
import io.grpc.StatusException
import io.grpc.android.UdsChannelBuilder
import java.net.InetAddress
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.asExecutor
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import mullvad_daemon.management_interface.ManagementInterface
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.LogInterceptor
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.connectivityFlow
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.AddSplitTunnelingAppError
import net.mullvad.mullvadvpn.model.AppId
import net.mullvad.mullvadvpn.model.AppVersionInfo as ModelAppVersionInfo
import net.mullvad.mullvadvpn.model.ClearAllOverridesError
import net.mullvad.mullvadvpn.model.ConnectError
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CreateAccountError
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.DeleteCustomListError
import net.mullvad.mullvadvpn.model.DeleteDeviceError
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.DeviceState as ModelDeviceState
import net.mullvad.mullvadvpn.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.model.GetAccountDataError
import net.mullvad.mullvadvpn.model.GetAccountHistoryError
import net.mullvad.mullvadvpn.model.GetDeviceListError
import net.mullvad.mullvadvpn.model.GetDeviceStateError
import net.mullvad.mullvadvpn.model.LoginAccountError
import net.mullvad.mullvadvpn.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.model.RedeemVoucherError
import net.mullvad.mullvadvpn.model.RedeemVoucherSuccess
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.model.RelayItemId as ModelRelayItemId
import net.mullvad.mullvadvpn.model.RelayList as ModelRelayList
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.RemoveSplitTunnelingAppError
import net.mullvad.mullvadvpn.model.SetAllowLanError
import net.mullvad.mullvadvpn.model.SetAutoConnectError
import net.mullvad.mullvadvpn.model.SetDnsOptionsError
import net.mullvad.mullvadvpn.model.SetObfuscationOptionsError
import net.mullvad.mullvadvpn.model.SetRelayLocationError
import net.mullvad.mullvadvpn.model.SetWireguardConstraintsError
import net.mullvad.mullvadvpn.model.SetWireguardMtuError
import net.mullvad.mullvadvpn.model.SetWireguardQuantumResistantError
import net.mullvad.mullvadvpn.model.Settings as ModelSettings
import net.mullvad.mullvadvpn.model.SettingsPatchError
import net.mullvad.mullvadvpn.model.TunnelState as ModelTunnelState
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import net.mullvad.mullvadvpn.model.WireguardConstraints as ModelWireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData as ModelWireguardEndpointData
import net.mullvad.mullvadvpn.model.WwwAuthToken
import net.mullvad.mullvadvpn.model.addresses
import net.mullvad.mullvadvpn.model.currentDnsOption
import net.mullvad.mullvadvpn.model.customOptions
import net.mullvad.mullvadvpn.model.location
import net.mullvad.mullvadvpn.model.ownership
import net.mullvad.mullvadvpn.model.providers
import net.mullvad.mullvadvpn.model.relayConstraints
import net.mullvad.mullvadvpn.model.wireguardConstraints

@Suppress("TooManyFunctions")
class ManagementService(
    rpcSocketPath: String,
    private val extensiveLogging: Boolean,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    private val channel =
        UdsChannelBuilder.forPath(rpcSocketPath, LocalSocketAddress.Namespace.FILESYSTEM).build()

    val connectionState: StateFlow<GrpcConnectivityState> =
        channel
            .connectivityFlow()
            .map(ConnectivityState::toDomain)
            .stateIn(scope, SharingStarted.Eagerly, channel.getState(false).toDomain())

    private val grpc =
        ManagementServiceGrpcKt.ManagementServiceCoroutineStub(channel)
            .withExecutor(Dispatchers.IO.asExecutor())
            .withInterceptors(
                if (extensiveLogging) {
                    LogInterceptor(TAG)
                } else {
                    null
                }
            )
            .withWaitForReady()

    private val _mutableDeviceState = MutableStateFlow<ModelDeviceState?>(null)
    val deviceState: Flow<ModelDeviceState> = _mutableDeviceState.filterNotNull()

    private val _mutableTunnelState = MutableStateFlow<ModelTunnelState?>(null)
    val tunnelState: Flow<ModelTunnelState> = _mutableTunnelState.filterNotNull()

    private val _mutableSettings = MutableStateFlow<ModelSettings?>(null)
    val settings: Flow<ModelSettings> = _mutableSettings.filterNotNull()

    private val _mutableVersionInfo = MutableStateFlow<ModelAppVersionInfo?>(null)
    val versionInfo: Flow<ModelAppVersionInfo> = _mutableVersionInfo.filterNotNull()

    private val _mutableRelayList = MutableStateFlow<RelayList?>(null)
    val relayList: Flow<RelayList> = _mutableRelayList.filterNotNull()

    val relayCountries: Flow<List<RelayItem.Location.Country>> =
        relayList.mapNotNull { it.countries }

    val wireguardEndpointData: Flow<ModelWireguardEndpointData> =
        relayList.mapNotNull { it.wireguardEndpointData }

    fun start() {
        // Just to ensure that connection is set up since the connection won't be setup without a
        // call to the daemon
        if (job != null) {
            error("ManagementService already started")
        }

        job = scope.launch { subscribeEvents() }
    }

    fun stop() {
        if (job == null) {
            error("ManagementService already stopped")
        }
        job!!.cancel(message = "ManagementService stopped")
        job = null
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    private suspend fun subscribeEvents() =
        withContext(Dispatchers.IO) {
            launch {
                grpc.eventsListen(Empty.getDefaultInstance()).collect { event ->
                    if (extensiveLogging) {
                        Log.d(TAG, "Event: $event")
                    }
                    @Suppress("WHEN_ENUM_CAN_BE_NULL_IN_JAVA")
                    when (event.eventCase) {
                        ManagementInterface.DaemonEvent.EventCase.TUNNEL_STATE ->
                            _mutableTunnelState.update { event.tunnelState.toDomain() }
                        ManagementInterface.DaemonEvent.EventCase.SETTINGS ->
                            _mutableSettings.update { event.settings.toDomain() }
                        ManagementInterface.DaemonEvent.EventCase.RELAY_LIST ->
                            _mutableRelayList.update { event.relayList.toDomain() }
                        ManagementInterface.DaemonEvent.EventCase.VERSION_INFO ->
                            _mutableVersionInfo.update { event.versionInfo.toDomain() }
                        ManagementInterface.DaemonEvent.EventCase.DEVICE ->
                            _mutableDeviceState.update { event.device.newState.toDomain() }
                        ManagementInterface.DaemonEvent.EventCase.REMOVE_DEVICE -> {}
                        ManagementInterface.DaemonEvent.EventCase.EVENT_NOT_SET -> {}
                        ManagementInterface.DaemonEvent.EventCase.NEW_ACCESS_METHOD -> {}
                    }
                }
            }
            getInitialServiceState()
        }

    suspend fun getDevice(): Either<GetDeviceStateError, ModelDeviceState> =
        Either.catch { grpc.getDevice(Empty.getDefaultInstance()) }
            .map { it.toDomain() }
            .mapLeft { GetDeviceStateError.Unknown(it) }

    suspend fun getDeviceList(token: AccountToken): Either<GetDeviceListError, List<Device>> =
        Either.catch { grpc.listDevices(StringValue.of(token.value)) }
            .map { it.devicesList.map(ManagementInterface.Device::toDomain) }
            .mapLeft { GetDeviceListError.Unknown(it) }

    suspend fun removeDevice(
        token: AccountToken,
        deviceId: DeviceId
    ): Either<DeleteDeviceError, Unit> =
        Either.catch {
                grpc.removeDevice(
                    ManagementInterface.DeviceRemoval.newBuilder()
                        .setAccountToken(token.value)
                        .setDeviceId(deviceId.value.toString())
                        .build(),
                )
            }
            .mapEmpty()
            .mapLeft { DeleteDeviceError.Unknown(it) }

    suspend fun connect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.connectTunnel(Empty.getDefaultInstance()).value }
            .mapLeft(ConnectError::Unknown)

    suspend fun disconnect(): Boolean = grpc.disconnectTunnel(Empty.getDefaultInstance()).value

    suspend fun reconnect(): Boolean = grpc.reconnectTunnel(Empty.getDefaultInstance()).value

    private suspend fun getTunnelState(): ModelTunnelState =
        grpc.getTunnelState(Empty.getDefaultInstance()).toDomain()

    private suspend fun getSettings(): ModelSettings =
        grpc.getSettings(Empty.getDefaultInstance()).toDomain()

    private suspend fun getDeviceState(): ModelDeviceState =
        grpc.getDevice(Empty.getDefaultInstance()).toDomain()

    private suspend fun getRelayList(): ModelRelayList =
        grpc.getRelayLocations(Empty.getDefaultInstance()).toDomain()

    private suspend fun getVersionInfo(): ModelAppVersionInfo =
        grpc.getVersionInfo(Empty.getDefaultInstance()).toDomain()

    suspend fun logoutAccount(): Unit {
        grpc.logoutAccount(Empty.getDefaultInstance())
    }

    suspend fun loginAccount(accountToken: AccountToken): Either<LoginAccountError, Unit> =
        Either.catch { grpc.loginAccount(StringValue.of(accountToken.value)) }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.UNAUTHENTICATED -> LoginAccountError.InvalidAccount
                    Status.Code.RESOURCE_EXHAUSTED ->
                        LoginAccountError.MaxDevicesReached(accountToken)
                    Status.Code.UNAVAILABLE -> LoginAccountError.RpcError
                    else -> LoginAccountError.Unknown(it)
                }
            }
            .mapEmpty()

    suspend fun clearAccountHistory(): Unit {
        grpc.clearAccountHistory(Empty.getDefaultInstance())
    }

    suspend fun getAccountHistory(): Either<GetAccountHistoryError, AccountToken?> =
        Either.catch {
                val history = grpc.getAccountHistory(Empty.getDefaultInstance())
                if (history.hasToken()) {
                    AccountToken(history.token.value)
                } else {
                    null
                }
            }
            .mapLeftStatus { GetAccountHistoryError.Unknown(it) }

    private suspend fun getInitialServiceState() {
        withContext(Dispatchers.IO) {
            awaitAll(
                async { _mutableTunnelState.update { getTunnelState() } },
                async { _mutableDeviceState.update { getDeviceState() } },
                async { _mutableSettings.update { getSettings() } },
                async { _mutableVersionInfo.update { getVersionInfo() } },
                async { _mutableRelayList.update { getRelayList() } },
            )
        }
    }

    suspend fun getAccountData(
        accountToken: AccountToken
    ): Either<GetAccountDataError, AccountData> =
        Either.catch { grpc.getAccountData(StringValue.of(accountToken.value)).toDomain() }
            .mapLeft { GetAccountDataError.Unknown(it) }

    suspend fun createAccount(): Either<CreateAccountError, AccountToken> =
        Either.catch {
                val accountTokenStringValue = grpc.createNewAccount(Empty.getDefaultInstance())
                AccountToken(accountTokenStringValue.value)
            }
            .mapLeft { CreateAccountError.Unknown(it) }

    suspend fun setDnsOptions(dnsOptions: ModelDnsOptions): Either<SetDnsOptionsError, Unit> =
        Either.catch { grpc.setDnsOptions(dnsOptions.fromDomain()) }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setDnsState(dnsState: ModelDnsState): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updated = DnsOptions.currentDnsOption.set(currentDnsOptions, dnsState)
                grpc.setDnsOptions(updated.fromDomain())
            }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setCustomDns(index: Int, address: InetAddress): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updatedDnsOptions =
                    DnsOptions.customOptions.addresses
                        .index(Index.list(), index)
                        .set(currentDnsOptions, address)

                grpc.setDnsOptions(updatedDnsOptions.fromDomain())
            }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun addCustomDns(address: InetAddress): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updatedDnsOptions =
                    DnsOptions.customOptions.addresses.modify(currentDnsOptions) { it + address }
                grpc.setDnsOptions(updatedDnsOptions.fromDomain())
            }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun deleteCustomDns(address: InetAddress): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updatedDnsOptions =
                    DnsOptions.customOptions.addresses.modify(currentDnsOptions) { it - address }
                grpc.setDnsOptions(updatedDnsOptions.fromDomain())
            }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setWireguardMtu(value: Int): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.setWireguardMtu(UInt32Value.of(value)) }
            .mapLeft(SetWireguardMtuError::Unknown)
            .mapEmpty()

    suspend fun resetWireguardMtu(): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.setWireguardMtu(UInt32Value.newBuilder().clearValue().build()) }
            .mapLeft(SetWireguardMtuError::Unknown)
            .mapEmpty()

    suspend fun setWireguardQuantumResistant(
        value: ModelQuantumResistantState
    ): Either<SetWireguardQuantumResistantError, Unit> =
        Either.catch { grpc.setQuantumResistantTunnel(value.toDomain()) }
            .mapLeft(SetWireguardQuantumResistantError::Unknown)
            .mapEmpty()

    // Todo needs to be more advanced
    suspend fun setRelaySettings(value: RelaySettings) {
        grpc.setRelaySettings(value.fromDomain())
    }

    suspend fun setObfuscationOptions(
        value: ModelObfuscationSettings
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch { grpc.setObfuscationSettings(value.fromDomain()) }
            .mapLeft(SetObfuscationOptionsError::Unknown)
            .mapEmpty()

    suspend fun setAutoConnect(isEnabled: Boolean): Either<SetAutoConnectError, Unit> =
        Either.catch { grpc.setAutoConnect(BoolValue.of(isEnabled)) }
            .mapLeft(SetAutoConnectError::Unknown)
            .mapEmpty()

    suspend fun setAllowLan(allow: Boolean): Either<SetAllowLanError, Unit> =
        Either.catch { grpc.setAllowLan(BoolValue.of(allow)) }
            .mapLeft(SetAllowLanError::Unknown)
            .mapEmpty()

    suspend fun getCurrentVersion(): String =
        grpc.getCurrentVersion(Empty.getDefaultInstance()).value

    suspend fun setRelayLocation(location: ModelRelayItemId): Either<SetRelayLocationError, Unit> =
        Either.catch {
                val currentRelaySettings = getSettings().relaySettings
                val updatedRelaySettings =
                    RelaySettings.relayConstraints.location.set(
                        currentRelaySettings,
                        Constraint.Only(location),
                    )
                grpc.setRelaySettings(updatedRelaySettings.fromDomain())
            }
            .mapLeft(SetRelayLocationError::Unknown)
            .mapEmpty()

    suspend fun createCustomList(
        name: CustomListName
    ): Either<CreateCustomListError, CustomListId> =
        Either.catch { grpc.createCustomList(StringValue.of(name.value)) }
            .map { CustomListId(it.value) }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.ALREADY_EXISTS -> CreateCustomListError.CustomListAlreadyExists
                    else -> CreateCustomListError.Unknown(it)
                }
            }

    suspend fun updateCustomList(customList: ModelCustomList): Either<UpdateCustomListError, Unit> =
        Either.catch { grpc.updateCustomList(customList.fromDomain()) }
            .mapLeft(UpdateCustomListError::Unknown)
            .mapEmpty()

    suspend fun deleteCustomList(id: CustomListId): Either<DeleteCustomListError, Unit> =
        Either.catch { grpc.deleteCustomList(StringValue.of(id.value)) }
            .mapLeft(DeleteCustomListError::Unknown)
            .mapEmpty()

    suspend fun clearAllRelayOverrides(): Either<ClearAllOverridesError, Unit> =
        Either.catch { grpc.clearAllRelayOverrides(Empty.getDefaultInstance()) }
            .mapLeft(ClearAllOverridesError::Unknown)
            .mapEmpty()

    suspend fun applySettingsPatch(json: String): Either<SettingsPatchError, Unit> =
        Either.catch { grpc.applyJsonSettings(StringValue.of(json)) }
            .mapLeftStatus {
                when (it.status.code) {
                    // Currently we only get invalid argument errors from daemon via gRPC
                    Status.Code.INVALID_ARGUMENT -> SettingsPatchError.ParsePatch
                    else -> SettingsPatchError.ApplyPatch
                }
            }
            .mapEmpty()

    suspend fun setWireguardConstraints(
        value: ModelWireguardConstraints
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.set(relaySettings, value)
                grpc.setRelaySettings(updated.fromDomain())
            }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setOwnershipAndProviders(
        ownershipConstraint: Constraint<ModelOwnership>,
        providersConstraint: Constraint<Providers>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    relaySettings.copy {
                        inside(RelaySettings.relayConstraints) {
                            RelayConstraints.providers set providersConstraint
                            RelayConstraints.ownership set ownershipConstraint
                        }
                    }
                grpc.setRelaySettings(updated.fromDomain())
            }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setOwnership(
        ownership: Constraint<ModelOwnership>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated = RelaySettings.relayConstraints.ownership.set(relaySettings, ownership)
                grpc.setRelaySettings(updated.fromDomain())
            }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setProviders(
        providersConstraint: Constraint<Providers>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.providers.set(relaySettings, providersConstraint)
                grpc.setRelaySettings(updated.fromDomain())
            }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun submitVoucher(voucher: String): Either<RedeemVoucherError, RedeemVoucherSuccess> =
        Either.catch { grpc.submitVoucher(StringValue.of(voucher)).toDomain() }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.INVALID_ARGUMENT,
                    Status.Code.NOT_FOUND -> RedeemVoucherError.InvalidVoucher
                    Status.Code.ALREADY_EXISTS,
                    Status.Code.RESOURCE_EXHAUSTED -> RedeemVoucherError.VoucherAlreadyUsed
                    Status.Code.UNAVAILABLE -> RedeemVoucherError.RpcError
                    else -> RedeemVoucherError.Unknown(it)
                }
            }

    suspend fun initializePlayPurchase(): Either<PlayPurchaseInitError, PlayPurchasePaymentToken> =
        Either.catch { grpc.initPlayPurchase(Empty.getDefaultInstance()).toDomain() }
            .mapLeft { PlayPurchaseInitError.OtherError }

    suspend fun verifyPlayPurchase(purchase: PlayPurchase): Either<PlayPurchaseVerifyError, Unit> =
        Either.catch { grpc.verifyPlayPurchase(purchase.fromDomain()) }
            .mapLeft { PlayPurchaseVerifyError.OtherError }
            .mapEmpty()

    suspend fun addSplitTunnelingApp(app: AppId): Either<AddSplitTunnelingAppError, Unit> =
        Either.catch { grpc.addSplitTunnelApp(StringValue.of(app.value)) }
            .mapLeft(AddSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun removeSplitTunnelingApp(app: AppId): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.removeSplitTunnelApp(StringValue.of(app.value)) }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun setSplitTunnelingState(
        enabled: Boolean
    ): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.setSplitTunnelState(BoolValue.of(enabled)) }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun getWwwAuthToken(): Either<Throwable, WwwAuthToken> =
        Either.catch { grpc.getWwwAuthToken(Empty.getDefaultInstance()) }
            .map { it.value.toDomain() }

    private fun <A> Either<A, Empty>.mapEmpty() = map {}

    private inline fun <B, C> Either<Throwable, B>.mapLeftStatus(
        f: (StatusException) -> C
    ): Either<C, B> = mapLeft {
        if (it is StatusException) {
            f(it)
        } else {
            throw it
        }
    }

    companion object {
        private const val TAG = "ManagementService"
    }
}

sealed interface GrpcConnectivityState {
    data object Connecting : GrpcConnectivityState

    data object Ready : GrpcConnectivityState

    data object Idle : GrpcConnectivityState

    data object TransientFailure : GrpcConnectivityState

    data object Shutdown : GrpcConnectivityState
}
