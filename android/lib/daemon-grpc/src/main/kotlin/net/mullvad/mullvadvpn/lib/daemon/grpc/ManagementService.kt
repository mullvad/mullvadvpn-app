package net.mullvad.mullvadvpn.lib.daemon.grpc

import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import arrow.optics.copy
import arrow.optics.dsl.index
import arrow.optics.typeclasses.Index
import co.touchlab.kermit.Logger
import com.squareup.wire.GrpcClient
import java.io.File
import java.net.InetAddress
import java.util.Map.entry
import java.util.concurrent.TimeUnit
import java.util.logging.Level
import java.util.logging.Logger as JavaLogger
import kotlin.getValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import mullvad_daemon.management_interface.BoolValue
import mullvad_daemon.management_interface.Device
import mullvad_daemon.management_interface.DeviceRemoval
import mullvad_daemon.management_interface.ManagementServiceClient
import mullvad_daemon.management_interface.NewCustomList
import mullvad_daemon.management_interface.StringValue
import mullvad_daemon.management_interface.UInt32Value
import net.mullvad.mullvadvpn.lib.daemon.grpc.mapper.fromDomain
import net.mullvad.mullvadvpn.lib.daemon.grpc.mapper.toDomain
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.AndroidLoggingHandler
import net.mullvad.mullvadvpn.lib.daemon.grpc.util.UnixDomainSocketFactory
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
import net.mullvad.mullvadvpn.lib.model.Device as ModelDevice
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState as ModelDeviceState
import net.mullvad.mullvadvpn.lib.model.DeviceUpdateError
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
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
import net.mullvad.mullvadvpn.lib.model.UpdateRelayLocationsError
import net.mullvad.mullvadvpn.lib.model.VoucherCode
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData as ModelWireguardEndpointData
import net.mullvad.mullvadvpn.lib.model.addresses
import net.mullvad.mullvadvpn.lib.model.customOptions
import net.mullvad.mullvadvpn.lib.model.defaultOptions
import net.mullvad.mullvadvpn.lib.model.entryLocation
import net.mullvad.mullvadvpn.lib.model.ipVersion
import net.mullvad.mullvadvpn.lib.model.isMultihopEnabled
import net.mullvad.mullvadvpn.lib.model.location
import net.mullvad.mullvadvpn.lib.model.ownership
import net.mullvad.mullvadvpn.lib.model.providers
import net.mullvad.mullvadvpn.lib.model.relayConstraints
import net.mullvad.mullvadvpn.lib.model.selectedObfuscationMode
import net.mullvad.mullvadvpn.lib.model.shadowsocks
import net.mullvad.mullvadvpn.lib.model.state
import net.mullvad.mullvadvpn.lib.model.udp2tcp
import net.mullvad.mullvadvpn.lib.model.wireguardConstraints
import net.mullvad.mullvadvpn.lib.model.wireguardPort
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.logging.HttpLoggingInterceptor
import org.newsclub.net.unix.AFUNIXSocketAddress

@Suppress("TooManyFunctions", "LargeClass")
class ManagementService(
    rpcSocketFile: File,
    private val extensiveLogging: Boolean,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    // We expect daemon to create the rpc socket file on the path provided on initialisation
    @Suppress("DEPRECATION") private val channel = socketClient(rpcSocketFile)

    val connectionState: StateFlow<GrpcConnectivityState> =
        channel
            .connectivityFlow()
            .onEach { Logger.i("ManagementService connection state: $it") }
            .stateIn(scope, SharingStarted.Eagerly, GrpcConnectivityState.Ready)

    private val grpc: ManagementServiceClient by lazy {
        GrpcClient.Builder()
            .client(channel)
            .baseUrl("http://10.0.2.2:3000/")
            .minMessageToCompress(Long.MAX_VALUE)
            .build()
            .create(ManagementServiceClient::class)
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

    fun enterIdle() = { /*NO-OP*/ }

    @Suppress("DEPRECATION")
    private suspend fun subscribeEvents() =
        withContext(Dispatchers.IO) {
            launch {
                val channel = grpc.EventsListen().executeIn(this, Unit)
                // channels.first.send(Unit)
                /*launch {
                    while(true) {
                        getInitialServiceState()
                        delay(100)
                    }
                }*/
                /*launch {
                    val c = grpc.EventsListen().executeBlocking(request = Unit)
                    Logger.d("LOLZ R ${c.read()}")
                }*/
                // launch { Logger.d("LOLZ ${channel.receive()}") }
                channel.receiveAsFlow().collect { event ->
                    if (extensiveLogging) {
                        Logger.v("Event: $event")
                    }
                    @Suppress("WHEN_ENUM_CAN_BE_NULL_IN_JAVA")
                    when {
                        event.tunnel_state != null ->
                            _mutableTunnelState.update { event.tunnel_state.toDomain() }
                        event.settings != null ->
                            _mutableSettings.update { event.settings.toDomain() }
                        event.relay_list != null ->
                            _mutableRelayList.update { event.relay_list.toDomain() }
                        event.version_info != null ->
                            _mutableVersionInfo.update { event.version_info.toDomain() }
                        event.device != null && event.device.new_state != null ->
                            _mutableDeviceState.update { event.device.new_state.toDomain() }
                        event.new_access_method != null -> {
                            _mutableCurrentAccessMethod.update {
                                event.new_access_method.toDomain()
                            }
                        }
                        event.remove_device != null -> {}
                    }
                }
            }
            getInitialServiceState()
        }

    suspend fun getDevice(): Either<GetDeviceStateError, ModelDeviceState> =
        Either.catch { grpc.GetDevice().execute(Unit) }
            .map { it.toDomain() }
            .onLeft { Logger.e("Get device error") }
            .mapLeft { GetDeviceStateError.Unknown(it) }

    suspend fun updateDevice(): Either<DeviceUpdateError, Unit> =
        Either.catch { grpc.UpdateDevice().execute(Unit) }
            .onLeft { Logger.e("Update device error") }
            .mapLeft { DeviceUpdateError(it) }

    suspend fun getDeviceList(token: AccountNumber): Either<GetDeviceListError, List<ModelDevice>> =
        Either.catch { grpc.ListDevices().execute(token.value.toStringValue()) }
            .map { it.devices.map(Device::toDomain) }
            .onLeft { Logger.e("Get device list error") }
            .mapLeft { GetDeviceListError.Unknown(it) }

    suspend fun removeDevice(
        token: AccountNumber,
        deviceId: DeviceId,
    ): Either<DeleteDeviceError, Unit> =
        Either.catch {
                grpc
                    .RemoveDevice()
                    .execute(
                        DeviceRemoval(
                            account_number = token.value,
                            device_id = deviceId.value.toString(),
                        )
                    )
            }
            .onLeft { Logger.e("Remove device error") }
            .mapLeft { DeleteDeviceError.Unknown(it) }

    suspend fun connect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.ConnectTunnel().execute(Unit).toBool() }
            .onLeft { Logger.e("Connect error") }
            .mapLeft(ConnectError::Unknown)

    suspend fun disconnect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.DisconnectTunnel().execute(Unit).toBool() }
            .onLeft { Logger.e("Disconnect error") }
            .mapLeft(ConnectError::Unknown)

    suspend fun reconnect(): Either<ConnectError, Boolean> =
        Either.catch { grpc.ReconnectTunnel().execute(Unit).toBool() }
            .onLeft { Logger.e("Reconnect error") }
            .mapLeft(ConnectError::Unknown)

    private suspend fun getTunnelState(): ModelTunnelState =
        grpc.GetTunnelState().execute(Unit).toDomain()

    private suspend fun getSettings(): ModelSettings = grpc.GetSettings().execute(Unit).toDomain()

    private suspend fun getDeviceState(): ModelDeviceState =
        grpc.GetDevice().execute(Unit).toDomain()

    private suspend fun getRelayList(): ModelRelayList =
        grpc.GetRelayLocations().execute(Unit).toDomain()

    // On release build this will return error until services have published the new beta, daemon
    // will get 404 until the api have been published, thus we need to ignore error downstream.
    private suspend fun getVersionInfo(): Either<GetVersionInfoError, ModelAppVersionInfo> =
        Either.catch { grpc.GetVersionInfo().execute(Unit).toDomain() }
            .onLeft { Logger.e("Get version info error") }
            .mapLeft { GetVersionInfoError.Unknown(it) }

    private suspend fun getCurrentApiAccessMethod(): ApiAccessMethodSetting =
        grpc.GetCurrentApiAccessMethod().execute(Unit).toDomain()

    suspend fun logoutAccount(): Either<LogoutAccountError, Unit> =
        Either.catch { grpc.LogoutAccount().execute(Unit) }
            .onLeft { Logger.e("Logout account error") }
            .mapLeft(LogoutAccountError::Unknown)

    suspend fun loginAccount(accountNumber: AccountNumber): Either<LoginAccountError, Unit> =
        Either.catch { grpc.LoginAccount().execute(accountNumber.value.toStringValue()) }
            .mapLeftStatus {
                /*when (it.status.code) {
                    Status.Code.UNAUTHENTICATED -> LoginAccountError.InvalidAccount
                    Status.Code.RESOURCE_EXHAUSTED if it.status.isTooManyRequests() ->
                        LoginAccountError.TooManyAttempts
                    Status.Code.RESOURCE_EXHAUSTED ->
                        LoginAccountError.MaxDevicesReached(accountNumber)
                    Status.Code.DEADLINE_EXCEEDED -> LoginAccountError.Timeout
                    Status.Code.INVALID_ARGUMENT -> LoginAccountError.InvalidInput(accountNumber)
                    Status.Code.UNAVAILABLE -> LoginAccountError.ApiUnreachable
                    else -> {
                        Logger.e("Unknown login account error")
                        LoginAccountError.Unknown(it)
                    }
                }*/
                LoginAccountError.InvalidAccount
            }

    suspend fun clearAccountHistory(): Either<ClearAccountHistoryError, Unit> =
        Either.catch { grpc.ClearAccountHistory().execute(Unit) }
            .onLeft { Logger.e("Clear account history error") }
            .mapLeft(ClearAccountHistoryError::Unknown)

    suspend fun getAccountHistory(): Either<GetAccountHistoryError, AccountNumber?> =
        Either.catch {
                val history = grpc.GetAccountHistory().execute(Unit)
                if (history.number != null) {
                    AccountNumber(history.number.value_)
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
                //async { _mutableVersionInfo.update { getVersionInfo().getOrNull() } },
                async { _mutableRelayList.update { getRelayList() } },
                async { _mutableCurrentAccessMethod.update { getCurrentApiAccessMethod() } },
            )
        }
    }

    suspend fun getAccountData(
        accountNumber: AccountNumber
    ): Either<GetAccountDataError, AccountData> =
        Either.catch {
                grpc
                    .GetAccountData()
                    .execute(accountNumber.value.toStringValue())
                    .toDomain(accountNumber)
            }
            .onLeft { Logger.e("Get account data error") }
            .mapLeft(GetAccountDataError::Unknown)

    suspend fun createAccount(): Either<CreateAccountError, AccountNumber> =
        Either.catch {
                val accountNumberStringValue = grpc.CreateNewAccount().execute(Unit)
                AccountNumber(accountNumberStringValue.value_)
            }
            .onLeft { Logger.e("Create account error") }
            .mapLeftStatus {
                /*when (it.status.code) {
                    Status.Code.RESOURCE_EXHAUSTED -> CreateAccountError.TooManyAttempts
                    Status.Code.UNAVAILABLE -> CreateAccountError.ApiUnreachable
                    Status.Code.DEADLINE_EXCEEDED -> CreateAccountError.TimeOut
                    else -> {
                        CreateAccountError.Unknown(it)
                    }
                }*/
                CreateAccountError.TooManyAttempts
            }

    suspend fun updateDnsContentBlockers(
        update: (DefaultDnsOptions) -> DefaultDnsOptions
    ): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val newDefaultDnsOptions = update(currentDnsOptions.defaultOptions)
                val updated = DnsOptions.defaultOptions.set(currentDnsOptions, newDefaultDnsOptions)
                grpc.SetDnsOptions().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set dns state error") }
            .mapLeft(SetDnsOptionsError::Unknown)

    suspend fun setDnsOptions(dnsOptions: ModelDnsOptions): Either<SetDnsOptionsError, Unit> =
        Either.catch { grpc.SetDnsOptions().execute(dnsOptions.fromDomain()) }
            .onLeft { Logger.e("Set dns options error") }
            .mapLeft(SetDnsOptionsError::Unknown)

    suspend fun setDnsState(dnsState: ModelDnsState): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updated = DnsOptions.state.set(currentDnsOptions, dnsState)
                grpc.SetDnsOptions().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set dns state error") }
            .mapLeft(SetDnsOptionsError::Unknown)

    suspend fun setCustomDns(index: Int, address: InetAddress): Either<SetDnsOptionsError, Unit> =
        Either.catch {
                val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
                val updatedDnsOptions =
                    DnsOptions.customOptions.addresses
                        .index(Index.list(), index)
                        .set(currentDnsOptions, address)

                grpc.SetDnsOptions().execute(updatedDnsOptions.fromDomain())
            }
            .onLeft { Logger.e("Set custom dns error") }
            .mapLeft(SetDnsOptionsError::Unknown)

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
                grpc.SetDnsOptions().execute(updatedDnsOptions.fromDomain())
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
                grpc.SetDnsOptions().execute(updatedDnsOptions.fromDomain())
            }
            .onLeft { Logger.e("Delete custom dns error") }
            .mapLeft(SetDnsOptionsError::Unknown)

    suspend fun setWireguardMtu(value: Int): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.SetWireguardMtu().execute(value.toUInt32Value()) }
            .onLeft { Logger.e("Set wireguard mtu error") }
            .mapLeft(SetWireguardMtuError::Unknown)

    suspend fun resetWireguardMtu(): Either<SetWireguardMtuError, Unit> =
        Either.catch { grpc.SetWireguardMtu().execute(UInt32Value()) }
            .onLeft { Logger.e("Reset wireguard mtu error") }
            .mapLeft(SetWireguardMtuError::Unknown)

    suspend fun setWireguardQuantumResistant(
        value: ModelQuantumResistantState
    ): Either<SetWireguardQuantumResistantError, Unit> =
        Either.catch { grpc.SetQuantumResistantTunnel().execute(value.fromDomain()) }
            .onLeft { Logger.e("Set wireguard quantum resistant error") }
            .mapLeft(SetWireguardQuantumResistantError::Unknown)

    suspend fun setObfuscation(value: ObfuscationMode): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedObfuscationSettings =
                    ObfuscationSettings.selectedObfuscationMode.modify(
                        getSettings().obfuscationSettings
                    ) {
                        value
                    }
                grpc.SetObfuscationSettings().execute(updatedObfuscationSettings.fromDomain())
            }
            .onLeft { Logger.e("Set obfuscation error") }
            .mapLeft(SetObfuscationOptionsError::Unknown)

    suspend fun setWireguardObfuscationPort(
        portConstraint: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedSettings =
                    ObfuscationSettings.wireguardPort.modify(getSettings().obfuscationSettings) {
                        portConstraint
                    }
                grpc.SetObfuscationSettings().execute(updatedSettings.fromDomain())
            }
            .onLeft { Logger.e("Set wireguard port error") }
            .mapLeft(SetObfuscationOptionsError::Unknown)

    suspend fun setUdp2TcpObfuscationPort(
        portConstraint: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedSettings =
                    ObfuscationSettings.udp2tcp.modify(getSettings().obfuscationSettings) {
                        it.copy(port = portConstraint)
                    }
                grpc.SetObfuscationSettings().execute(updatedSettings.fromDomain())
            }
            .onLeft { Logger.e("Set obfuscation port error") }
            .mapLeft(SetObfuscationOptionsError::Unknown)

    suspend fun setShadowsocksObfuscationPort(
        portConstraint: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        Either.catch {
                val updatedSettings =
                    ObfuscationSettings.shadowsocks.modify(getSettings().obfuscationSettings) {
                        it.copy(port = portConstraint)
                    }
                grpc.SetObfuscationSettings().execute(updatedSettings.fromDomain())
            }
            .mapLeft(SetObfuscationOptionsError::Unknown)

    suspend fun setAllowLan(allow: Boolean): Either<SetAllowLanError, Unit> =
        Either.catch { grpc.SetAllowLan().execute(allow.toBoolValue()) }
            .onLeft { Logger.e("Set allow lan error") }
            .mapLeft(SetAllowLanError::Unknown)

    suspend fun setDaitaEnabled(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.SetEnableDaita().execute(enabled.toBoolValue()) }
            .mapLeft(SetDaitaSettingsError::Unknown)

    suspend fun setDaitaDirectOnly(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.SetDaitaDirectOnly().execute(enabled.toBoolValue()) }
            .mapLeft(SetDaitaSettingsError::Unknown)

    suspend fun setRelayLocation(location: ModelRelayItemId): Either<SetRelayLocationError, Unit> =
        Either.catch {
                val currentRelaySettings = getSettings().relaySettings
                val updatedRelaySettings =
                    RelaySettings.relayConstraints.location.set(
                        currentRelaySettings,
                        Constraint.Only(location),
                    )
                grpc.SetRelaySettings().execute(updatedRelaySettings.fromDomain())
            }
            .onLeft { Logger.e("Set relay location error") }
            .mapLeft(SetRelayLocationError::Unknown)

    suspend fun setRelayLocationMultihop(
        isMultihopEnabled: Boolean,
        entry: RelayItemId?,
        exit: RelayItemId,
    ): Either<SetRelayLocationError, Unit> =
        Either.catch {
                val currentRelaySettings = getSettings().relaySettings

                val updatedRelaySettings =
                    currentRelaySettings.copy {
                        inside(RelaySettings.relayConstraints) {
                            RelayConstraints.location set Constraint.Only(exit)
                            if (entry != null) {
                                RelayConstraints.wireguardConstraints.entryLocation set
                                    Constraint.Only(entry)
                            }
                            RelayConstraints.wireguardConstraints.isMultihopEnabled set
                                isMultihopEnabled
                        }
                    }
                grpc.SetRelaySettings().execute(updatedRelaySettings.fromDomain())
            }
            .onLeft { Logger.e("Set relay multihop error") }
            .mapLeft(SetRelayLocationError::Unknown)

    suspend fun createCustomList(
        name: CustomListName,
        locations: List<GeoLocationId> = emptyList(),
    ): Either<CreateCustomListError, CustomListId> =
        Either.catch {
                grpc
                    .CreateCustomList()
                    .execute(
                        NewCustomList(
                            name = name.value,
                            locations = locations.map(GeoLocationId::fromDomain),
                        )
                    )
            }
            .map { CustomListId(it.value_) }
            .mapLeftStatus {
                /*when (it.status.code) {
                    Status.Code.ALREADY_EXISTS -> CustomListAlreadyExists
                    else -> {
                        Logger.e("Unknown create custom list error")
                        UnknownCustomListError(it)
                    }
                }*/
                CustomListAlreadyExists
            }

    suspend fun updateCustomList(customList: ModelCustomList): Either<UpdateCustomListError, Unit> =
        Either.catch { grpc.UpdateCustomList().execute(customList.fromDomain()) }
            .mapLeftStatus {
                /*when (it.status.code) {
                    Status.Code.ALREADY_EXISTS -> NameAlreadyExists(customList.name)
                    else -> {
                        Logger.e("Unknown update custom list error")
                        UnknownCustomListError(it)
                    }
                }*/
                NameAlreadyExists(customList.name)
            }

    suspend fun deleteCustomList(id: CustomListId): Either<DeleteCustomListError, Unit> =
        Either.catch { grpc.DeleteCustomList().execute(id.value.toStringValue()) }
            .onLeft { Logger.e("Delete custom list error") }
            .mapLeft(::UnknownCustomListError)

    suspend fun clearAllRelayOverrides(): Either<ClearAllOverridesError, Unit> =
        Either.catch { grpc.ClearAllRelayOverrides().execute(Unit) }
            .onLeft { Logger.e("Clear all relay overrides error") }
            .mapLeft(ClearAllOverridesError::Unknown)

    suspend fun applySettingsPatch(json: String): Either<SettingsPatchError, Unit> =
        Either.catch { grpc.ApplyJsonSettings().execute(json.toStringValue()) }
            .mapLeftStatus {
                /*when (it.status.code) {
                    // Currently we only get invalid argument errors from daemon via gRPC
                    Status.Code.INVALID_ARGUMENT -> SettingsPatchError.ParsePatch
                    else -> {
                        Logger.e("Unknown apply settings patch error")
                        SettingsPatchError.ApplyPatch
                    }
                }*/
                SettingsPatchError.ApplyPatch
            }

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
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set ownership and providers error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    suspend fun setOwnership(
        ownership: Constraint<ModelOwnership>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated = RelaySettings.relayConstraints.ownership.set(relaySettings, ownership)
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set ownership error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    suspend fun setProviders(
        providersConstraint: Constraint<Providers>
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.providers.set(relaySettings, providersConstraint)
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set providers error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    suspend fun submitVoucher(
        voucher: VoucherCode
    ): Either<RedeemVoucherError, RedeemVoucherSuccess> =
        Either.catch { grpc.SubmitVoucher().execute(voucher.value.toStringValue()).toDomain() }
            .mapLeftStatus {
                /*when (it.status.code) {
                    Status.Code.INVALID_ARGUMENT,
                    Status.Code.NOT_FOUND -> RedeemVoucherError.InvalidVoucher
                    Status.Code.ALREADY_EXISTS,
                    Status.Code.RESOURCE_EXHAUSTED -> RedeemVoucherError.VoucherAlreadyUsed
                    Status.Code.UNAVAILABLE -> RedeemVoucherError.ApiUnreachable
                    else -> {
                        Logger.e("Unknown submit voucher error")
                        RedeemVoucherError.Unknown(it)
                    }
                }*/
                RedeemVoucherError.Unknown(Throwable())
            }

    suspend fun initializePlayPurchase(): Either<PlayPurchaseInitError, PlayPurchasePaymentToken> =
        Either.catch { grpc.InitPlayPurchase().execute(Unit).toDomain() }
            .onLeft { Logger.e("Initialize play purchase error") }
            .mapLeft { PlayPurchaseInitError.OtherError }

    suspend fun verifyPlayPurchase(purchase: PlayPurchase): Either<PlayPurchaseVerifyError, Unit> =
        Either.catch { grpc.VerifyPlayPurchase().execute(purchase.fromDomain()) }
            .onLeft { Logger.e("Verify play purchase error") }
            .mapLeft { PlayPurchaseVerifyError.OtherError }

    suspend fun addSplitTunnelingApp(app: AppId): Either<AddSplitTunnelingAppError, Unit> =
        Either.catch { grpc.AddSplitTunnelApp().execute(app.value.toStringValue()) }
            .onLeft { Logger.e("Add split tunneling app error") }
            .mapLeft(AddSplitTunnelingAppError::Unknown)

    suspend fun removeSplitTunnelingApp(app: AppId): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.RemoveSplitTunnelApp().execute(app.value.toStringValue()) }
            .onLeft { Logger.e("Remove split tunneling app error") }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)

    suspend fun setSplitTunnelingState(
        enabled: Boolean
    ): Either<RemoveSplitTunnelingAppError, Unit> =
        Either.catch { grpc.SetSplitTunnelState().execute(enabled.toBoolValue()) }
            .onLeft { Logger.e("Set split tunneling state error") }
            .mapLeft(RemoveSplitTunnelingAppError::Unknown)

    suspend fun getWebsiteAuthToken(): Either<Throwable, WebsiteAuthToken> =
        Either.catch { grpc.GetWwwAuthToken().execute(Unit) }
            .onLeft { Logger.e("Get website auth token error") }
            .map { WebsiteAuthToken.fromString(it.value_) }

    suspend fun addApiAccessMethod(
        newAccessMethodSetting: NewAccessMethodSetting
    ): Either<AddApiAccessMethodError, ApiAccessMethodId> =
        Either.catch { grpc.AddApiAccessMethod().execute(newAccessMethodSetting.fromDomain()) }
            .onLeft { Logger.e("Add api access method error") }
            .mapLeft(AddApiAccessMethodError::Unknown)
            .map { ApiAccessMethodId.fromString(it.value_) }

    suspend fun removeApiAccessMethod(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<RemoveApiAccessMethodError, Unit> =
        Either.catch { grpc.RemoveApiAccessMethod().execute(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Remove api access method error") }
            .mapLeft(RemoveApiAccessMethodError::Unknown)

    suspend fun setApiAccessMethod(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<SetApiAccessMethodError, Unit> =
        Either.catch { grpc.SetApiAccessMethod().execute(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Set api access method error") }
            .mapLeft(SetApiAccessMethodError::Unknown)

    suspend fun updateApiAccessMethod(
        apiAccessMethodSetting: ApiAccessMethodSetting
    ): Either<UpdateApiAccessMethodError, Unit> =
        Either.catch { grpc.UpdateApiAccessMethod().execute(apiAccessMethodSetting.fromDomain()) }
            .onLeft { Logger.e("Update api access method error") }
            .mapLeft(::UnknownApiAccessMethodError)

    suspend fun testCustomApiAccessMethod(
        customProxy: ApiAccessMethod.CustomProxy
    ): Either<TestApiAccessMethodError, Unit> =
        Either.catch { grpc.TestCustomApiAccessMethod().execute(customProxy.fromDomain()) }
            .onLeft { Logger.e("Test custom api access method error") }
            .mapLeftStatus { TestApiAccessMethodError.Grpc }
            .map { result ->
                either { ensure(result.value_) { TestApiAccessMethodError.CouldNotAccess } }
            }

    suspend fun testApiAccessMethodById(
        apiAccessMethodId: ApiAccessMethodId
    ): Either<TestApiAccessMethodError, Unit> =
        Either.catch { grpc.TestApiAccessMethodById().execute(apiAccessMethodId.fromDomain()) }
            .onLeft { Logger.e("Test api access method error") }
            .mapLeftStatus { TestApiAccessMethodError.Grpc }
            .map { result ->
                either { ensure(result.value_) { TestApiAccessMethodError.CouldNotAccess } }
            }

    suspend fun setMultihop(enabled: Boolean): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val relaySettings = getSettings().relaySettings
                val updated =
                    RelaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled.set(
                        relaySettings,
                        enabled,
                    )
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

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
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

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
                grpc.SetRelaySettings().execute(updated.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    suspend fun setIpv6Enabled(enabled: Boolean): Either<SetDaitaSettingsError, Unit> =
        Either.catch { grpc.SetEnableIpv6().execute(enabled.toBoolValue()) }
            .mapLeft(SetDaitaSettingsError::Unknown)

    suspend fun setRecentsEnabled(enabled: Boolean): Either<SetWireguardConstraintsError, Unit> =
        Either.catch { grpc.SetEnableRecents().execute(enabled.toBoolValue()) }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    suspend fun updateRelayLocations(): Either<UpdateRelayLocationsError, Unit> =
        Either.catch { grpc.UpdateRelayLocations().execute(Unit) }
            .mapLeft(UpdateRelayLocationsError::Unknown)

    suspend fun setMultihopAndEntryLocation(
        isMultihopEnabled: Boolean,
        entryLocation: RelayItemId,
    ): Either<SetWireguardConstraintsError, Unit> =
        Either.catch {
                val currentRelaySettings = getSettings().relaySettings
                val updatedRelaySettings =
                    currentRelaySettings.copy {
                        inside(RelaySettings.relayConstraints.wireguardConstraints) {
                            WireguardConstraints.entryLocation set Constraint.Only(entryLocation)
                            WireguardConstraints.isMultihopEnabled set isMultihopEnabled
                        }
                    }
                grpc.SetRelaySettings().execute(updatedRelaySettings.fromDomain())
            }
            .onLeft { Logger.e("Set multihop error") }
            .mapLeft(SetWireguardConstraintsError::Unknown)

    private fun Boolean.toBoolValue() = BoolValue(this)

    private fun String.toStringValue() = StringValue(this)

    private fun Int.toUInt32Value() = UInt32Value(this)

    private fun BoolValue.toBool() = this.value_

    /*private inline fun <B, C> Either<Throwable, B>.mapLeftStatus(
        f: (StatusException) -> C
    ): Either<C, B> = mapLeft {
        if (it is StatusException) {
            f(it)
        } else {
            throw it
        }
    }*/

    private inline fun <B, C> Either<Throwable, B>.mapLeftStatus(f: (Any) -> C): Either<C, B> =
        mapLeft {
            f(it)
        }

    private fun socketClient(rpcSocketFile: File): OkHttpClient {
        val addr = AFUNIXSocketAddress.of(rpcSocketFile)

        return OkHttpClient.Builder()
            //.socketFactory(socketFactory = UnixDomainSocketFactory(addr))
            // .socketFactory(socketFactory =
            // AFSocketFactory.FixedAddressSocketFactory(rpcSocketFile))
            // .callTimeout(java.time.Duration.ofMinutes(1))
            .callTimeout(timeout = 0, TimeUnit.MILLISECONDS)
            .readTimeout(timeout = 0, TimeUnit.MILLISECONDS)
            .connectTimeout(timeout = 0, TimeUnit.MILLISECONDS)
            .writeTimeout(timeout = 0, TimeUnit.MILLISECONDS)
            .webSocketCloseTimeout(timeout = 0, TimeUnit.MILLISECONDS)
            .protocols(listOf(Protocol.H2_PRIOR_KNOWLEDGE))
            .addInterceptor(
                HttpLoggingInterceptor { message -> Logger.withTag("grpc").d(message) }
                    .also { it.level = HttpLoggingInterceptor.Level.BODY }
            )
            .build()
    }

    // private fun Status.isTooManyRequests() = description == TOO_MANY_REQUESTS

    companion object {
        const val ENABLE_TRACE_LOGGING = false

        const val TOO_MANY_REQUESTS = "429 Too Many Requests"
    }
}

sealed interface GrpcConnectivityState {
    data object Connecting : GrpcConnectivityState

    data object Ready : GrpcConnectivityState

    data object Idle : GrpcConnectivityState

    data object TransientFailure : GrpcConnectivityState

    data object Shutdown : GrpcConnectivityState
}
