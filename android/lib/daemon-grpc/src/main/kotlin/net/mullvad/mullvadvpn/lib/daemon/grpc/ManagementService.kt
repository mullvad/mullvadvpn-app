package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.LocalSocketAddress
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.optics.copy
import arrow.optics.dsl.index
import arrow.optics.typeclasses.Index
import co.touchlab.kermit.Logger
import com.google.protobuf.BoolValue
import com.google.protobuf.Empty
import com.google.protobuf.StringValue
import com.google.protobuf.UInt32Value
import io.grpc.ConnectivityState
import io.grpc.Status
import io.grpc.StatusException
import io.grpc.android.UdsChannelBuilder
import java.io.File
import java.net.InetAddress
import java.util.logging.Level
import java.util.logging.Logger as JavaLogger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
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
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import mullvad_daemon.management_interface.ManagementInterface
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
import net.mullvad.mullvadvpn.lib.daemon.grpc.mapper.fromDomain
import net.mullvad.mullvadvpn.lib.daemon.grpc.mapper.toDomain
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.AndroidLoggingHandler
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.LogInterceptor
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.connectivityFlow
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.AddApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.AddSplitTunnelingAppError
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.AppVersionInfo as ModelAppVersionInfo
import net.mullvad.mullvadvpn.lib.model.ClearAccountHistoryError
import net.mullvad.mullvadvpn.lib.model.ClearAllOverridesError
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CreateAccountError
import net.mullvad.mullvadvpn.lib.model.CreateCustomListError
import net.mullvad.mullvadvpn.lib.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.lib.model.CustomListAlreadyExists
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DeleteCustomListError
import net.mullvad.mullvadvpn.lib.model.DeleteDeviceError
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState as ModelDeviceState
import net.mullvad.mullvadvpn.lib.model.DeviceUpdateError
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.GetAccountDataError
import net.mullvad.mullvadvpn.lib.model.GetAccountHistoryError
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.model.GetDeviceStateError
import net.mullvad.mullvadvpn.lib.model.GetVersionInfoError
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.model.LogoutAccountError
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherError
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherSuccess
import net.mullvad.mullvadvpn.lib.model.RelayConstraints
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId as ModelRelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayList as ModelRelayList
import net.mullvad.mullvadvpn.lib.model.RelayList
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.RemoveApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.RemoveSplitTunnelingAppError
import net.mullvad.mullvadvpn.lib.model.SetAllowLanError
import net.mullvad.mullvadvpn.lib.model.SetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.SetDaitaSettingsError
import net.mullvad.mullvadvpn.lib.model.SetDnsOptionsError
import net.mullvad.mullvadvpn.lib.model.SetObfuscationOptionsError
import net.mullvad.mullvadvpn.lib.model.SetRelayLocationError
import net.mullvad.mullvadvpn.lib.model.SetWireguardConstraintsError
import net.mullvad.mullvadvpn.lib.model.SetWireguardMtuError
import net.mullvad.mullvadvpn.lib.model.SetWireguardQuantumResistantError
import net.mullvad.mullvadvpn.lib.model.Settings as ModelSettings
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.TunnelState as ModelTunnelState
import net.mullvad.mullvadvpn.lib.model.UnknownApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.UnknownCustomListError
import net.mullvad.mullvadvpn.lib.model.UpdateApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.UpdateCustomListError
import net.mullvad.mullvadvpn.lib.model.VoucherCode
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData as ModelWireguardEndpointData
import net.mullvad.mullvadvpn.lib.model.addresses
import net.mullvad.mullvadvpn.lib.model.customOptions
import net.mullvad.mullvadvpn.lib.model.defaultOptions
import net.mullvad.mullvadvpn.lib.model.entryLocation
import net.mullvad.mullvadvpn.lib.model.ipVersion
import net.mullvad.mullvadvpn.lib.model.isMultihopEnabled
import net.mullvad.mullvadvpn.lib.model.location
import net.mullvad.mullvadvpn.lib.model.ownership
import net.mullvad.mullvadvpn.lib.model.port
import net.mullvad.mullvadvpn.lib.model.providers
import net.mullvad.mullvadvpn.lib.model.relayConstraints
import net.mullvad.mullvadvpn.lib.model.selectedObfuscationMode
import net.mullvad.mullvadvpn.lib.model.shadowsocks
import net.mullvad.mullvadvpn.lib.model.state
import net.mullvad.mullvadvpn.lib.model.udp2tcp
import net.mullvad.mullvadvpn.lib.model.wireguardConstraints

@Suppress("TooManyFunctions", "LargeClass")
class ManagementService(
    rpcSocketFile: File,
    private val extensiveLogging: Boolean,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    // We expect daemon to create the rpc socket file on the path provided on initialisation
    @Suppress("DEPRECATION")
    private val channel =
        UdsChannelBuilder.forPath(
                rpcSocketFile.absolutePath,
                LocalSocketAddress.Namespace.FILESYSTEM,
            )
            // Workaround for handling WiFi with proxy
            // https://github.com/grpc/grpc-java/issues/11922
            .proxyDetector { null }
            .build()

    val connectionState: StateFlow<GrpcConnectivityState> =
        channel
            .connectivityFlow()
            .map(ConnectivityState::toDomain)
            .onEach { Logger.i("ManagementService connection state: $it") }
            .stateIn(scope, SharingStarted.Eagerly, channel.getState(false).toDomain())

    private val grpc by lazy {
        ManagementServiceGrpcKt.ManagementServiceCoroutineStub(channel)
            .withExecutor(Dispatchers.IO.asExecutor())
            .let {
                if (extensiveLogging) {
                    it.withInterceptors(LogInterceptor())
                } else it
            }
            .withWaitForReady()
    }

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

    private val _mutableCurrentAccessMethod = MutableStateFlow<ApiAccessMethodSetting?>(null)
    val currentAccessMethod: Flow<ApiAccessMethodSetting> =
        _mutableCurrentAccessMethod.filterNotNull()

    init {
        if (extensiveLogging && ENABLE_TRACE_LOGGING) {
            AndroidLoggingHandler.reset(AndroidLoggingHandler())
            JavaLogger.getLogger("io.grpc").level = Level.FINEST
        }
    }

    fun start() {
        // Just to ensure that connection is set up since the connection won't be setup without a
        // call to the daemon
        if (job != null) {
            error("ManagementService already started")
        }

        job = scope.launch { subscribeEvents() }
    }

    fun stop() {
        job?.cancel(message = "ManagementService stopped")
            ?: error("ManagementService already stopped")
        job = null
    }

    fun enterIdle() = channel.enterIdle()

    private suspend fun subscribeEvents() =
        withContext(Dispatchers.IO) {
            launch {
                grpc.eventsListen(Empty.getDefaultInstance()).collect { event ->
                    if (extensiveLogging) {
                        Logger.v("Event: $event")
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
                        ManagementInterface.DaemonEvent.EventCase.NEW_ACCESS_METHOD -> {
                            _mutableCurrentAccessMethod.update { event.newAccessMethod.toDomain() }
                        }
                        ManagementInterface.DaemonEvent.EventCase.REMOVE_DEVICE -> {}
                        ManagementInterface.DaemonEvent.EventCase.EVENT_NOT_SET -> {}
                    }
                }
            }
            getInitialServiceState()
        }

    suspend fun getDevice(): Either<GetDeviceStateError, ModelDeviceState> =
        Either.catch { grpc.getDevice(Empty.getDefaultInstance()) }
            .map { it.toDomain() }
            .onLeft { Logger.e("Get device error") }
            .mapLeft { GetDeviceStateError.Unknown(it) }

    suspend fun updateDevice(): Either<DeviceUpdateError, Unit> =
        Either.catch { grpc.updateDevice(Empty.getDefaultInstance()) }
            .mapEmpty()
            .onLeft { Logger.e("Update device error") }
            .mapLeft { DeviceUpdateError(it) }

    suspend fun getDeviceList(token: AccountNumber): Either<GetDeviceListError, List<Device>> =
        Either.catch { grpc.listDevices(StringValue.of(token.value)) }
            .map { it.devicesList.map(ManagementInterface.Device::toDomain) }
            .onLeft { Logger.e("Get device list error") }
            .mapLeft { GetDeviceListError.Unknown(it) }

    suspend fun removeDevice(
        token: AccountNumber,
        deviceId: DeviceId,
    ): Either<DeleteDeviceError, Unit> =
        Either.catch {
                grpc.removeDevice(
                    ManagementInterface.DeviceRemoval.newBuilder()
                        .setAccountNumber(token.value)
                        .setDeviceId(deviceId.value.toString())
                        .build()
                )
            }
            .mapEmpty()
            .onLeft { Logger.e("Remove device error") }
            .mapLeft { DeleteDeviceError.Unknown(it) }

    suspend fun connect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.connectTunnel(Empty.getDefaultInstance()).value }
            .onLeft { Logger.e("Connect error") }
            .mapLeft(ConnectError::Unknown)

    suspend fun disconnect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.disconnectTunnel(Empty.getDefaultInstance()).value }
            .onLeft { Logger.e("Disconnect error") }
            .mapLeft(ConnectError::Unknown)

    suspend fun reconnect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.reconnectTunnel(Empty.getDefaultInstance()).value }
            .onLeft { Logger.e("Reconnect error") }
            .mapLeft(ConnectError::Unknown)

    private suspend fun getTunnelState(): ModelTunnelState =
        grpc.getTunnelState(Empty.getDefaultInstance()).toDomain()

    private suspend fun getSettings(): ModelSettings =
        grpc.getSettings(Empty.getDefaultInstance()).toDomain()

    private suspend fun getDeviceState(): ModelDeviceState =
        grpc.getDevice(Empty.getDefaultInstance()).toDomain()

    private suspend fun getRelayList(): ModelRelayList =
        grpc.getRelayLocations(Empty.getDefaultInstance()).toDomain()

    // On release build this will return error until services have published the new beta, daemon
    // will get 404 until the api have been published, thus we need to ignore error downstream.
    private suspend fun getVersionInfo(): Either<GetVersionInfoError, ModelAppVersionInfo> =
        Either.catch { grpc.getVersionInfo(Empty.getDefaultInstance()).toDomain() }
            .onLeft { Logger.e("Get version info error") }
            .mapLeft { GetVersionInfoError.Unknown(it) }

    private suspend fun getCurrentApiAccessMethod(): ApiAccessMethodSetting =
        grpc.getCurrentApiAccessMethod(Empty.getDefaultInstance()).toDomain()

    suspend fun logoutAccount(): Either<LogoutAccountError, Unit> =
        Either.catch { grpc.logoutAccount(Empty.getDefaultInstance()) }
            .onLeft { Logger.e("Logout account error") }
            .mapLeft(LogoutAccountError::Unknown)
            .mapEmpty()

    suspend fun loginAccount(accountNumber: AccountNumber): Either<LoginAccountError, Unit> =
        Either.catch { grpc.loginAccount(StringValue.of(accountNumber.value)) }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.UNAUTHENTICATED -> LoginAccountError.InvalidAccount
                    Status.Code.RESOURCE_EXHAUSTED ->
                        LoginAccountError.MaxDevicesReached(accountNumber)
                    Status.Code.UNAVAILABLE -> LoginAccountError.RpcError
                    else -> {
                        Logger.e("Unknown login account error")
                        LoginAccountError.Unknown(it)
                    }
                }
            }
            .mapEmpty()

    suspend fun clearAccountHistory(): Either<ClearAccountHistoryError, Unit> =
        Either.catch { grpc.clearAccountHistory(Empty.getDefaultInstance()) }
            .onLeft { Logger.e("Clear account history error") }
            .mapLeft(ClearAccountHistoryError::Unknown)
            .mapEmpty()

    suspend fun getAccountHistory(): Either<GetAccountHistoryError, AccountNumber?> =
        Either.catch {
                val history = grpc.getAccountHistory(Empty.getDefaultInstance())
                if (history.hasNumber()) {
                    AccountNumber(history.number.value)
                } else {
                    null
                }
            }
            .onLeft { Logger.e("Get account history error") }
            .mapLeft(GetAccountHistoryError::Unknown)

    private suspend fun getInitialServiceState() {
        withContext(Dispatchers.IO) {
            awaitAll(
                async { _mutableTunnelState.update { getTunnelState() } },
                async { _mutableDeviceState.update { getDeviceState() } },
                async { _mutableSettings.update { getSettings() } },
                async { _mutableVersionInfo.update { getVersionInfo().getOrNull() } },
                async { _mutableRelayList.update { getRelayList() } },
                async { _mutableCurrentAccessMethod.update { getCurrentApiAccessMethod() } },
            )
        }
    }

    suspend fun getAccountData(
        accountNumber: AccountNumber
    ): Either<GetAccountDataError, AccountData> =
        Either.catch { grpc.getAccountData(StringValue.of(accountNumber.value)).toDomain() }
            .onLeft { Logger.e("Get account data error") }
            .mapLeft(GetAccountDataError::Unknown)

    suspend fun createAccount(): Either<CreateAccountError, AccountNumber> =
        Either.catch {
                val accountNumberStringValue = grpc.createNewAccount(Empty.getDefaultInstance())
                AccountNumber(accountNumberStringValue.value)
            }
            .onLeft { Logger.e("Create account error") }
            .mapLeft(CreateAccountError::Unknown)

    suspend fun updateDnsContentBlockers(
        update: (DefaultDnsOptions) -> DefaultDnsOptions
    ): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val newDefaultDnsOptions = update(currentDnsOptions.defaultOptions)
                val updated = DnsOptions.defaultOptions.set(currentDnsOptions, newDefaultDnsOptions)
                grpc.setDnsOptions(updated.fromDomain())
            }
            .onLeft { Logger.e("Set dns state error") }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setDnsOptions(dnsOptions: ModelDnsOptions): Either<SetDnsOptionsError, Unit> =
        Either.catch { grpc.setDnsOptions(dnsOptions.fromDomain()) }
            .onLeft { Logger.e("Set dns options error") }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setDnsState(dnsState: ModelDnsState): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updated = DnsOptions.state.set(currentDnsOptions, dnsState)
                grpc.setDnsOptions(updated.fromDomain())
            }
            .onLeft { Logger.e("Set dns state error") }
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
            .onLeft { Logger.e("Set custom dns error") }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun addCustomDns(address: InetAddress): Either<SetDnsOptionsError, Int> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updatedDnsOptions =
                    currentDnsOptions.copy {
                        DnsOptions.customOptions.addresses set
                            currentDnsOptions.customOptions.addresses + address
                        // If it is the first address, then turn on Custom Dns
                        DnsOptions.state set
                            if (currentDnsOptions.customOptions.addresses.isEmpty()) DnsState.Custom
                            else currentDnsOptions.state
                    }
                grpc.setDnsOptions(updatedDnsOptions.fromDomain())
                updatedDnsOptions.customOptions.addresses.lastIndex
            }
            .onLeft { Logger.e("Add custom dns error") }
            .mapLeft(SetDnsOptionsError::Unknown)

    suspend fun deleteCustomDns(index: Int): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val mutableAddresses = currentDnsOptions.customOptions.addresses.toMutableList()
                mutableAddresses.removeAt(index)

                val updatedDnsOptions =
                    currentDnsOptions.copy {
                        DnsOptions.customOptions.addresses set mutableAddresses.toList()
                        // If it is the last address, then turn off Custom Dns
                        DnsOptions.state set
                            if (mutableAddresses.isEmpty()) DnsState.Default
                            else currentDnsOptions.state
                    }
                grpc.setDnsOptions(updatedDnsOptions.fromDomain())
            }
            .onLeft { Logger.e("Delete custom dns error") }
            .mapLeft(SetDnsOptionsError::Unknown)
            .mapEmpty()

    suspend fun setWireguardMtu(value: Int): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.setWireguardMtu(UInt32Value.of(value)) }
            .onLeft { Logger.e("Set wireguard mtu error") }
            .mapLeft(SetWireguardMtuError::Unknown)
            .mapEmpty()

    suspend fun resetWireguardMtu(): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.setWireguardMtu(UInt32Value.newBuilder().clearValue().build()) }
            .onLeft { Logger.e("Reset wireguard mtu error") }
            .mapLeft(SetWireguardMtuError::Unknown)
            .mapEmpty()

    suspend fun setWireguardQuantumResistant(
        value: ModelQuantumResistantState
    ): Either<SetWireguardQuantumResistantError, Unit> =
        Either.catch { grpc.setQuantumResistantTunnel(value.toDomain()) }
            .onLeft { Logger.e("Set wireguard quantum resistant error") }
            .mapLeft(SetWireguardQuantumResistantError::Unknown)
            .mapEmpty()

    suspend fun setObfuscation(value: ObfuscationMode): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedObfuscationSettings =
                    ObfuscationSettings.selectedObfuscationMode.modify(
                        getSettings().obfuscationSettings
                    ) {
                        value
                    }
                grpc.setObfuscationSettings(updatedObfuscationSettings.fromDomain())
            }
            .onLeft { Logger.e("Set obfuscation error") }
            .mapLeft(SetObfuscationOptionsError::Unknown)
            .mapEmpty()

    suspend fun setUdp2TcpObfuscationPort(
        portConstraint: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedSettings =
                    ObfuscationSettings.udp2tcp.modify(getSettings().obfuscationSettings) {
                        it.copy(port = portConstraint)
                    }
                grpc.setObfuscationSettings(updatedSettings.fromDomain())
            }
            .onLeft { Logger.e("Set obfuscation port error") }
            .mapLeft(SetObfuscationOptionsError::Unknown)
            .mapEmpty()

    suspend fun setShadowsocksObfuscationPort(
        portConstraint: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedSettings =
                    ObfuscationSettings.shadowsocks.modify(getSettings().obfuscationSettings) {
                        it.copy(port = portConstraint)
                    }
                grpc.setObfuscationSettings(updatedSettings.fromDomain())
            }
            .mapLeft(SetObfuscationOptionsError::Unknown)
            .mapEmpty()

    suspend fun setAllowLan(allow: Boolean): Either<SetAllowLanError, Unit> =
        Either.catch { grpc.setAllowLan(BoolValue.of(allow)) }
            .onLeft { Logger.e("Set allow lan error") }
            .mapLeft(SetAllowLanError::Unknown)
            .mapEmpty()

    suspend fun setDaitaEnabled(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.setEnableDaita(BoolValue.of(enabled)) }
            .mapLeft(SetDaitaSettingsError::Unknown)
            .mapEmpty()

    suspend fun setDaitaDirectOnly(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.setDaitaDirectOnly(BoolValue.of(enabled)) }
            .mapLeft(SetDaitaSettingsError::Unknown)
            .mapEmpty()

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
            .onLeft { Logger.e("Set relay location error") }
            .mapLeft(SetRelayLocationError::Unknown)
            .mapEmpty()

    suspend fun createCustomList(
        name: CustomListName
    ): Either<CreateCustomListError, CustomListId> =
        Either.catch { grpc.createCustomList(StringValue.of(name.value)) }
            .map { CustomListId(it.value) }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.ALREADY_EXISTS -> CustomListAlreadyExists
                    else -> {
                        Logger.e("Unknown create custom list error")
                        UnknownCustomListError(it)
                    }
                }
            }

    suspend fun updateCustomList(customList: ModelCustomList): Either<UpdateCustomListError, Unit> =
        Either.catch { grpc.updateCustomList(customList.fromDomain()) }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.ALREADY_EXISTS -> NameAlreadyExists(customList.name)
                    else -> {
                        Logger.e("Unknown update custom list error")
                        UnknownCustomListError(it)
                    }
                }
            }
            .mapEmpty()

    suspend fun deleteCustomList(id: CustomListId): Either<DeleteCustomListError, Unit> =
        Either.catch { grpc.deleteCustomList(StringValue.of(id.value)) }
            .onLeft { Logger.e("Delete custom list error") }
            .mapLeft(::UnknownCustomListError)
            .mapEmpty()

    suspend fun clearAllRelayOverrides(): Either<ClearAllOverridesError, Unit> =
        Either.catch { grpc.clearAllRelayOverrides(Empty.getDefaultInstance()) }
            .onLeft { Logger.e("Clear all relay overrides error") }
            .mapLeft(ClearAllOverridesError::Unknown)
            .mapEmpty()

    suspend fun applySettingsPatch(json: String): Either<SettingsPatchError, Unit> =
        Either.catch { grpc.applyJsonSettings(StringValue.of(json)) }
            .mapLeftStatus {
                when (it.status.code) {
                    // Currently we only get invalid argument errors from daemon via gRPC
                    Status.Code.INVALID_ARGUMENT -> SettingsPatchError.ParsePatch
                    else -> {
                        Logger.e("Unknown apply settings patch error")
                        SettingsPatchError.ApplyPatch
                    }
                }
            }
            .mapEmpty()

    suspend fun setOwnershipAndProviders(
        ownershipConstraint: Constraint<ModelOwnership>,
        providersConstraint: Constraint<Providers>,
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
            .onLeft { Logger.e("Set ownership and providers error") }
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
            .onLeft { Logger.e("Set ownership error") }
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
            .onLeft { Logger.e("Set providers error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun submitVoucher(
        voucher: VoucherCode
    ): Either<RedeemVoucherError, RedeemVoucherSuccess> =
        Either.catch { grpc.submitVoucher(StringValue.of(voucher.value)).toDomain() }
            .mapLeftStatus {
                when (it.status.code) {
                    Status.Code.INVALID_ARGUMENT,
                    Status.Code.NOT_FOUND -> RedeemVoucherError.InvalidVoucher
                    Status.Code.ALREADY_EXISTS,
                    Status.Code.RESOURCE_EXHAUSTED -> RedeemVoucherError.VoucherAlreadyUsed
                    Status.Code.UNAVAILABLE -> RedeemVoucherError.RpcError
                    else -> {
                        Logger.e("Unknown submit voucher error")
                        RedeemVoucherError.Unknown(it)
                    }
                }
            }

    suspend fun initializePlayPurchase(): Either<PlayPurchaseInitError, PlayPurchasePaymentToken> =
        Either.catch { grpc.initPlayPurchase(Empty.getDefaultInstance()).toDomain() }
            .onLeft { Logger.e("Initialize play purchase error") }
            .mapLeft { PlayPurchaseInitError.OtherError }

    suspend fun verifyPlayPurchase(purchase: PlayPurchase): Either<PlayPurchaseVerifyError, Unit> =
        Either.catch { grpc.verifyPlayPurchase(purchase.fromDomain()) }
            .onLeft { Logger.e("Verify play purchase error") }
            .mapLeft { PlayPurchaseVerifyError.OtherError }
            .mapEmpty()

    suspend fun addSplitTunnelingApp(app: AppId): Either<AddSplitTunnelingAppError, Unit> =
        Either.catch { grpc.addSplitTunnelApp(StringValue.of(app.value)) }
            .onLeft { Logger.e("Add split tunneling app error") }
            .mapLeft(AddSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun removeSplitTunnelingApp(app: AppId): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.removeSplitTunnelApp(StringValue.of(app.value)) }
            .onLeft { Logger.e("Remove split tunneling app error") }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun setSplitTunnelingState(
        enabled: Boolean
    ): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.setSplitTunnelState(BoolValue.of(enabled)) }
            .onLeft { Logger.e("Set split tunneling state error") }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)
            .mapEmpty()

    suspend fun getWebsiteAuthToken(): Either<Throwable, WebsiteAuthToken> =
        Either.catch { grpc.getWwwAuthToken(Empty.getDefaultInstance()) }
            .onLeft { Logger.e("Get website auth token error") }
            .map { WebsiteAuthToken.fromString(it.value) }

    suspend fun addApiAccessMethod(
        newAccessMethodSetting: NewAccessMethodSetting
    ): Either<AddApiAccessMethodError, ApiAccessMethodId> =
        Either.catch { grpc.addApiAccessMethod(newAccessMethodSetting.fromDomain()) }
            .onLeft { Logger.e("Add api access method error") }
            .mapLeft(AddApiAccessMethodError::Unknown)
            .map { ApiAccessMethodId.fromString(it.value) }

    suspend fun removeApiAccessMethod(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<RemoveApiAccessMethodError, Unit> =
        Either.catch { grpc.removeApiAccessMethod(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Remove api access method error") }
            .mapLeft(RemoveApiAccessMethodError::Unknown)
            .mapEmpty()

    suspend fun setApiAccessMethod(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<SetApiAccessMethodError, Unit> =
        Either.catch { grpc.setApiAccessMethod(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Set api access method error") }
            .mapLeft(SetApiAccessMethodError::Unknown)
            .mapEmpty()

    suspend fun updateApiAccessMethod(
        apiAccessMethodSetting: ApiAccessMethodSetting
    ): Either<UpdateApiAccessMethodError, Unit> =
        Either.catch { grpc.updateApiAccessMethod(apiAccessMethodSetting.fromDomain()) }
            .onLeft { Logger.e("Update api access method error") }
            .mapLeft(::UnknownApiAccessMethodError)
            .mapEmpty()

    suspend fun testCustomApiAccessMethod(
        customProxy: ApiAccessMethod.CustomProxy
    ): Either<TestApiAccessMethodError, Unit> =
        Either.catch { grpc.testCustomApiAccessMethod(customProxy.fromDomain()) }
            .onLeft { Logger.e("Test custom api access method error") }
            .mapLeftStatus { TestApiAccessMethodError.Grpc }
            .map { result ->
                either { ensure(result.value) { TestApiAccessMethodError.CouldNotAccess } }
            }

    suspend fun testApiAccessMethodById(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<TestApiAccessMethodError, Unit> =
        Either.catch { grpc.testApiAccessMethodById(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Test api access method error") }
            .mapLeftStatus { TestApiAccessMethodError.Grpc }
            .map { result ->
                either { ensure(result.value) { TestApiAccessMethodError.CouldNotAccess } }
            }

    suspend fun setWireguardPort(
        port: Constraint<Port>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.port.set(
                        relaySettings,
                        port,
                    )
                grpc.setRelaySettings(updated.fromDomain())
            }
            .onLeft { Logger.e("Set wireguard port error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setMultihop(enabled: Boolean): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled.set(
                        relaySettings,
                        enabled,
                    )
                grpc.setRelaySettings(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setEntryLocation(
        entryLocation: RelayItemId
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.entryLocation.set(
                        relaySettings,
                        Constraint.Only(entryLocation),
                    )
                grpc.setRelaySettings(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setDeviceIpVersion(
        ipVersion: Constraint<IpVersion>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.ipVersion.set(
                        relaySettings,
                        ipVersion,
                    )
                grpc.setRelaySettings(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)
            .mapEmpty()

    suspend fun setIpv6Enabled(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.setEnableIpv6(BoolValue.of(enabled)) }
            .mapLeft(SetDaitaSettingsError::Unknown)
            .mapEmpty()

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
        const val ENABLE_TRACE_LOGGING = false
    }
}

sealed interface GrpcConnectivityState {
    data object Connecting : GrpcConnectivityState

    data object Ready : GrpcConnectivityState

    data object Idle : GrpcConnectivityState

    data object TransientFailure : GrpcConnectivityState

    data object Shutdown : GrpcConnectivityState
}
