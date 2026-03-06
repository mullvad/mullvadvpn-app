// package: mullvad_daemon.management_interface
// file: management_interface.proto

/* tslint:disable */
/* eslint-disable */

import * as grpc from "@grpc/grpc-js";
import * as management_interface_pb from "./management_interface_pb";
import * as google_protobuf_empty_pb from "google-protobuf/google/protobuf/empty_pb";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";
import * as google_protobuf_wrappers_pb from "google-protobuf/google/protobuf/wrappers_pb";
import * as google_protobuf_duration_pb from "google-protobuf/google/protobuf/duration_pb";

interface IManagementServiceService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    connectTunnel: IManagementServiceService_IConnectTunnel;
    disconnectTunnel: IManagementServiceService_IDisconnectTunnel;
    reconnectTunnel: IManagementServiceService_IReconnectTunnel;
    getTunnelState: IManagementServiceService_IGetTunnelState;
    eventsListen: IManagementServiceService_IEventsListen;
    prepareRestart: IManagementServiceService_IPrepareRestart;
    prepareRestartV2: IManagementServiceService_IPrepareRestartV2;
    factoryReset: IManagementServiceService_IFactoryReset;
    getCurrentVersion: IManagementServiceService_IGetCurrentVersion;
    getVersionInfo: IManagementServiceService_IGetVersionInfo;
    isPerformingPostUpgrade: IManagementServiceService_IIsPerformingPostUpgrade;
    updateRelayLocations: IManagementServiceService_IUpdateRelayLocations;
    getRelayLocations: IManagementServiceService_IGetRelayLocations;
    setRelaySettings: IManagementServiceService_ISetRelaySettings;
    setObfuscationSettings: IManagementServiceService_ISetObfuscationSettings;
    getSettings: IManagementServiceService_IGetSettings;
    resetSettings: IManagementServiceService_IResetSettings;
    setAllowLan: IManagementServiceService_ISetAllowLan;
    setShowBetaReleases: IManagementServiceService_ISetShowBetaReleases;
    setLockdownMode: IManagementServiceService_ISetLockdownMode;
    setAutoConnect: IManagementServiceService_ISetAutoConnect;
    setWireguardMtu: IManagementServiceService_ISetWireguardMtu;
    setWireguardAllowedIps: IManagementServiceService_ISetWireguardAllowedIps;
    setEnableIpv6: IManagementServiceService_ISetEnableIpv6;
    setQuantumResistantTunnel: IManagementServiceService_ISetQuantumResistantTunnel;
    setEnableDaita: IManagementServiceService_ISetEnableDaita;
    setDaitaDirectOnly: IManagementServiceService_ISetDaitaDirectOnly;
    setDaitaSettings: IManagementServiceService_ISetDaitaSettings;
    setDnsOptions: IManagementServiceService_ISetDnsOptions;
    setRelayOverride: IManagementServiceService_ISetRelayOverride;
    clearAllRelayOverrides: IManagementServiceService_IClearAllRelayOverrides;
    setEnableRecents: IManagementServiceService_ISetEnableRecents;
    createNewAccount: IManagementServiceService_ICreateNewAccount;
    loginAccount: IManagementServiceService_ILoginAccount;
    logoutAccount: IManagementServiceService_ILogoutAccount;
    getAccountData: IManagementServiceService_IGetAccountData;
    getAccountHistory: IManagementServiceService_IGetAccountHistory;
    clearAccountHistory: IManagementServiceService_IClearAccountHistory;
    getWwwAuthToken: IManagementServiceService_IGetWwwAuthToken;
    submitVoucher: IManagementServiceService_ISubmitVoucher;
    getDevice: IManagementServiceService_IGetDevice;
    updateDevice: IManagementServiceService_IUpdateDevice;
    listDevices: IManagementServiceService_IListDevices;
    removeDevice: IManagementServiceService_IRemoveDevice;
    setWireguardRotationInterval: IManagementServiceService_ISetWireguardRotationInterval;
    resetWireguardRotationInterval: IManagementServiceService_IResetWireguardRotationInterval;
    rotateWireguardKey: IManagementServiceService_IRotateWireguardKey;
    getWireguardKey: IManagementServiceService_IGetWireguardKey;
    createCustomList: IManagementServiceService_ICreateCustomList;
    deleteCustomList: IManagementServiceService_IDeleteCustomList;
    updateCustomList: IManagementServiceService_IUpdateCustomList;
    clearCustomLists: IManagementServiceService_IClearCustomLists;
    addApiAccessMethod: IManagementServiceService_IAddApiAccessMethod;
    removeApiAccessMethod: IManagementServiceService_IRemoveApiAccessMethod;
    setApiAccessMethod: IManagementServiceService_ISetApiAccessMethod;
    updateApiAccessMethod: IManagementServiceService_IUpdateApiAccessMethod;
    clearCustomApiAccessMethods: IManagementServiceService_IClearCustomApiAccessMethods;
    getCurrentApiAccessMethod: IManagementServiceService_IGetCurrentApiAccessMethod;
    testCustomApiAccessMethod: IManagementServiceService_ITestCustomApiAccessMethod;
    testApiAccessMethodById: IManagementServiceService_ITestApiAccessMethodById;
    getBridges: IManagementServiceService_IGetBridges;
    getSplitTunnelProcesses: IManagementServiceService_IGetSplitTunnelProcesses;
    addSplitTunnelProcess: IManagementServiceService_IAddSplitTunnelProcess;
    removeSplitTunnelProcess: IManagementServiceService_IRemoveSplitTunnelProcess;
    clearSplitTunnelProcesses: IManagementServiceService_IClearSplitTunnelProcesses;
    splitTunnelIsSupported: IManagementServiceService_ISplitTunnelIsSupported;
    addSplitTunnelApp: IManagementServiceService_IAddSplitTunnelApp;
    removeSplitTunnelApp: IManagementServiceService_IRemoveSplitTunnelApp;
    setSplitTunnelState: IManagementServiceService_ISetSplitTunnelState;
    clearSplitTunnelApps: IManagementServiceService_IClearSplitTunnelApps;
    getExcludedProcesses: IManagementServiceService_IGetExcludedProcesses;
    initPlayPurchase: IManagementServiceService_IInitPlayPurchase;
    verifyPlayPurchase: IManagementServiceService_IVerifyPlayPurchase;
    needFullDiskPermissions: IManagementServiceService_INeedFullDiskPermissions;
    checkVolumes: IManagementServiceService_ICheckVolumes;
    applyJsonSettings: IManagementServiceService_IApplyJsonSettings;
    exportJsonSettings: IManagementServiceService_IExportJsonSettings;
    getFeatureIndicators: IManagementServiceService_IGetFeatureIndicators;
    disableRelay: IManagementServiceService_IDisableRelay;
    enableRelay: IManagementServiceService_IEnableRelay;
    getRolloutThreshold: IManagementServiceService_IGetRolloutThreshold;
    regenerateRolloutThreshold: IManagementServiceService_IRegenerateRolloutThreshold;
    setRolloutThresholdSeed: IManagementServiceService_ISetRolloutThresholdSeed;
    appUpgrade: IManagementServiceService_IAppUpgrade;
    appUpgradeAbort: IManagementServiceService_IAppUpgradeAbort;
    appUpgradeEventsListen: IManagementServiceService_IAppUpgradeEventsListen;
    getAppUpgradeCacheDir: IManagementServiceService_IGetAppUpgradeCacheDir;
    setLogFilter: IManagementServiceService_ISetLogFilter;
    logListen: IManagementServiceService_ILogListen;
}

interface IManagementServiceService_IConnectTunnel extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/ConnectTunnel";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IDisconnectTunnel extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/DisconnectTunnel";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IReconnectTunnel extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/ReconnectTunnel";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IGetTunnelState extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.TunnelState> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetTunnelState";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.TunnelState>;
    responseDeserialize: grpc.deserialize<management_interface_pb.TunnelState>;
}
interface IManagementServiceService_IEventsListen extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.DaemonEvent> {
    path: "/mullvad_daemon.management_interface.ManagementService/EventsListen";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.DaemonEvent>;
    responseDeserialize: grpc.deserialize<management_interface_pb.DaemonEvent>;
}
interface IManagementServiceService_IPrepareRestart extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/PrepareRestart";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IPrepareRestartV2 extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/PrepareRestartV2";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IFactoryReset extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/FactoryReset";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetCurrentVersion extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetCurrentVersion";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_IGetVersionInfo extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.AppVersionInfo> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetVersionInfo";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.AppVersionInfo>;
    responseDeserialize: grpc.deserialize<management_interface_pb.AppVersionInfo>;
}
interface IManagementServiceService_IIsPerformingPostUpgrade extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/IsPerformingPostUpgrade";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IUpdateRelayLocations extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/UpdateRelayLocations";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetRelayLocations extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.RelayList> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetRelayLocations";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.RelayList>;
    responseDeserialize: grpc.deserialize<management_interface_pb.RelayList>;
}
interface IManagementServiceService_ISetRelaySettings extends grpc.MethodDefinition<management_interface_pb.RelaySettings, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetRelaySettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.RelaySettings>;
    requestDeserialize: grpc.deserialize<management_interface_pb.RelaySettings>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetObfuscationSettings extends grpc.MethodDefinition<management_interface_pb.ObfuscationSettings, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetObfuscationSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.ObfuscationSettings>;
    requestDeserialize: grpc.deserialize<management_interface_pb.ObfuscationSettings>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetSettings extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.Settings> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.Settings>;
    responseDeserialize: grpc.deserialize<management_interface_pb.Settings>;
}
interface IManagementServiceService_IResetSettings extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ResetSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetAllowLan extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetAllowLan";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetShowBetaReleases extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetShowBetaReleases";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetLockdownMode extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetLockdownMode";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetAutoConnect extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetAutoConnect";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetWireguardMtu extends grpc.MethodDefinition<google_protobuf_wrappers_pb.UInt32Value, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetWireguardMtu";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.UInt32Value>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.UInt32Value>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetWireguardAllowedIps extends grpc.MethodDefinition<management_interface_pb.AllowedIpsList, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetWireguardAllowedIps";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.AllowedIpsList>;
    requestDeserialize: grpc.deserialize<management_interface_pb.AllowedIpsList>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetEnableIpv6 extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetEnableIpv6";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetQuantumResistantTunnel extends grpc.MethodDefinition<management_interface_pb.QuantumResistantState, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetQuantumResistantTunnel";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.QuantumResistantState>;
    requestDeserialize: grpc.deserialize<management_interface_pb.QuantumResistantState>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetEnableDaita extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetEnableDaita";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetDaitaDirectOnly extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetDaitaDirectOnly";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetDaitaSettings extends grpc.MethodDefinition<management_interface_pb.DaitaSettings, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetDaitaSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.DaitaSettings>;
    requestDeserialize: grpc.deserialize<management_interface_pb.DaitaSettings>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetDnsOptions extends grpc.MethodDefinition<management_interface_pb.DnsOptions, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetDnsOptions";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.DnsOptions>;
    requestDeserialize: grpc.deserialize<management_interface_pb.DnsOptions>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetRelayOverride extends grpc.MethodDefinition<management_interface_pb.RelayOverride, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetRelayOverride";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.RelayOverride>;
    requestDeserialize: grpc.deserialize<management_interface_pb.RelayOverride>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IClearAllRelayOverrides extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearAllRelayOverrides";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetEnableRecents extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetEnableRecents";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ICreateNewAccount extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/CreateNewAccount";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_ILoginAccount extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/LoginAccount";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ILogoutAccount extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/LogoutAccount";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetAccountData extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, management_interface_pb.AccountData> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetAccountData";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<management_interface_pb.AccountData>;
    responseDeserialize: grpc.deserialize<management_interface_pb.AccountData>;
}
interface IManagementServiceService_IGetAccountHistory extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.AccountHistory> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetAccountHistory";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.AccountHistory>;
    responseDeserialize: grpc.deserialize<management_interface_pb.AccountHistory>;
}
interface IManagementServiceService_IClearAccountHistory extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearAccountHistory";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetWwwAuthToken extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetWwwAuthToken";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_ISubmitVoucher extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, management_interface_pb.VoucherSubmission> {
    path: "/mullvad_daemon.management_interface.ManagementService/SubmitVoucher";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<management_interface_pb.VoucherSubmission>;
    responseDeserialize: grpc.deserialize<management_interface_pb.VoucherSubmission>;
}
interface IManagementServiceService_IGetDevice extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.DeviceState> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetDevice";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.DeviceState>;
    responseDeserialize: grpc.deserialize<management_interface_pb.DeviceState>;
}
interface IManagementServiceService_IUpdateDevice extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/UpdateDevice";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IListDevices extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, management_interface_pb.DeviceList> {
    path: "/mullvad_daemon.management_interface.ManagementService/ListDevices";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<management_interface_pb.DeviceList>;
    responseDeserialize: grpc.deserialize<management_interface_pb.DeviceList>;
}
interface IManagementServiceService_IRemoveDevice extends grpc.MethodDefinition<management_interface_pb.DeviceRemoval, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/RemoveDevice";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.DeviceRemoval>;
    requestDeserialize: grpc.deserialize<management_interface_pb.DeviceRemoval>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetWireguardRotationInterval extends grpc.MethodDefinition<google_protobuf_duration_pb.Duration, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetWireguardRotationInterval";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_duration_pb.Duration>;
    requestDeserialize: grpc.deserialize<google_protobuf_duration_pb.Duration>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IResetWireguardRotationInterval extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ResetWireguardRotationInterval";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IRotateWireguardKey extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/RotateWireguardKey";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetWireguardKey extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.PublicKey> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetWireguardKey";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.PublicKey>;
    responseDeserialize: grpc.deserialize<management_interface_pb.PublicKey>;
}
interface IManagementServiceService_ICreateCustomList extends grpc.MethodDefinition<management_interface_pb.NewCustomList, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/CreateCustomList";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.NewCustomList>;
    requestDeserialize: grpc.deserialize<management_interface_pb.NewCustomList>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_IDeleteCustomList extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/DeleteCustomList";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IUpdateCustomList extends grpc.MethodDefinition<management_interface_pb.CustomList, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/UpdateCustomList";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.CustomList>;
    requestDeserialize: grpc.deserialize<management_interface_pb.CustomList>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IClearCustomLists extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearCustomLists";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IAddApiAccessMethod extends grpc.MethodDefinition<management_interface_pb.NewAccessMethodSetting, management_interface_pb.UUID> {
    path: "/mullvad_daemon.management_interface.ManagementService/AddApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.NewAccessMethodSetting>;
    requestDeserialize: grpc.deserialize<management_interface_pb.NewAccessMethodSetting>;
    responseSerialize: grpc.serialize<management_interface_pb.UUID>;
    responseDeserialize: grpc.deserialize<management_interface_pb.UUID>;
}
interface IManagementServiceService_IRemoveApiAccessMethod extends grpc.MethodDefinition<management_interface_pb.UUID, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/RemoveApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.UUID>;
    requestDeserialize: grpc.deserialize<management_interface_pb.UUID>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetApiAccessMethod extends grpc.MethodDefinition<management_interface_pb.UUID, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.UUID>;
    requestDeserialize: grpc.deserialize<management_interface_pb.UUID>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IUpdateApiAccessMethod extends grpc.MethodDefinition<management_interface_pb.AccessMethodSetting, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/UpdateApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.AccessMethodSetting>;
    requestDeserialize: grpc.deserialize<management_interface_pb.AccessMethodSetting>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IClearCustomApiAccessMethods extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearCustomApiAccessMethods";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetCurrentApiAccessMethod extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.AccessMethodSetting> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetCurrentApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.AccessMethodSetting>;
    responseDeserialize: grpc.deserialize<management_interface_pb.AccessMethodSetting>;
}
interface IManagementServiceService_ITestCustomApiAccessMethod extends grpc.MethodDefinition<management_interface_pb.CustomProxy, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/TestCustomApiAccessMethod";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.CustomProxy>;
    requestDeserialize: grpc.deserialize<management_interface_pb.CustomProxy>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_ITestApiAccessMethodById extends grpc.MethodDefinition<management_interface_pb.UUID, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/TestApiAccessMethodById";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.UUID>;
    requestDeserialize: grpc.deserialize<management_interface_pb.UUID>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IGetBridges extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.BridgeList> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetBridges";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.BridgeList>;
    responseDeserialize: grpc.deserialize<management_interface_pb.BridgeList>;
}
interface IManagementServiceService_IGetSplitTunnelProcesses extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.Int32Value> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetSplitTunnelProcesses";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.Int32Value>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.Int32Value>;
}
interface IManagementServiceService_IAddSplitTunnelProcess extends grpc.MethodDefinition<google_protobuf_wrappers_pb.Int32Value, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/AddSplitTunnelProcess";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.Int32Value>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.Int32Value>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IRemoveSplitTunnelProcess extends grpc.MethodDefinition<google_protobuf_wrappers_pb.Int32Value, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/RemoveSplitTunnelProcess";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.Int32Value>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.Int32Value>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IClearSplitTunnelProcesses extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearSplitTunnelProcesses";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISplitTunnelIsSupported extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/SplitTunnelIsSupported";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_IAddSplitTunnelApp extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/AddSplitTunnelApp";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IRemoveSplitTunnelApp extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/RemoveSplitTunnelApp";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ISetSplitTunnelState extends grpc.MethodDefinition<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetSplitTunnelState";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IClearSplitTunnelApps extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ClearSplitTunnelApps";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetExcludedProcesses extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.ExcludedProcessList> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetExcludedProcesses";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.ExcludedProcessList>;
    responseDeserialize: grpc.deserialize<management_interface_pb.ExcludedProcessList>;
}
interface IManagementServiceService_IInitPlayPurchase extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.PlayPurchasePaymentToken> {
    path: "/mullvad_daemon.management_interface.ManagementService/InitPlayPurchase";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.PlayPurchasePaymentToken>;
    responseDeserialize: grpc.deserialize<management_interface_pb.PlayPurchasePaymentToken>;
}
interface IManagementServiceService_IVerifyPlayPurchase extends grpc.MethodDefinition<management_interface_pb.PlayPurchase, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/VerifyPlayPurchase";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.PlayPurchase>;
    requestDeserialize: grpc.deserialize<management_interface_pb.PlayPurchase>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_INeedFullDiskPermissions extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/NeedFullDiskPermissions";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.BoolValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.BoolValue>;
}
interface IManagementServiceService_ICheckVolumes extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/CheckVolumes";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IApplyJsonSettings extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/ApplyJsonSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IExportJsonSettings extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/ExportJsonSettings";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_IGetFeatureIndicators extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.FeatureIndicators> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetFeatureIndicators";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.FeatureIndicators>;
    responseDeserialize: grpc.deserialize<management_interface_pb.FeatureIndicators>;
}
interface IManagementServiceService_IDisableRelay extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/DisableRelay";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IEnableRelay extends grpc.MethodDefinition<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/EnableRelay";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    requestDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IGetRolloutThreshold extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.Rollout> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetRolloutThreshold";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.Rollout>;
    responseDeserialize: grpc.deserialize<management_interface_pb.Rollout>;
}
interface IManagementServiceService_IRegenerateRolloutThreshold extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.Rollout> {
    path: "/mullvad_daemon.management_interface.ManagementService/RegenerateRolloutThreshold";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.Rollout>;
    responseDeserialize: grpc.deserialize<management_interface_pb.Rollout>;
}
interface IManagementServiceService_ISetRolloutThresholdSeed extends grpc.MethodDefinition<management_interface_pb.Seed, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetRolloutThresholdSeed";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.Seed>;
    requestDeserialize: grpc.deserialize<management_interface_pb.Seed>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IAppUpgrade extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/AppUpgrade";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IAppUpgradeAbort extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/AppUpgradeAbort";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_IAppUpgradeEventsListen extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.AppUpgradeEvent> {
    path: "/mullvad_daemon.management_interface.ManagementService/AppUpgradeEventsListen";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.AppUpgradeEvent>;
    responseDeserialize: grpc.deserialize<management_interface_pb.AppUpgradeEvent>;
}
interface IManagementServiceService_IGetAppUpgradeCacheDir extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue> {
    path: "/mullvad_daemon.management_interface.ManagementService/GetAppUpgradeCacheDir";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<google_protobuf_wrappers_pb.StringValue>;
    responseDeserialize: grpc.deserialize<google_protobuf_wrappers_pb.StringValue>;
}
interface IManagementServiceService_ISetLogFilter extends grpc.MethodDefinition<management_interface_pb.LogFilter, google_protobuf_empty_pb.Empty> {
    path: "/mullvad_daemon.management_interface.ManagementService/SetLogFilter";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<management_interface_pb.LogFilter>;
    requestDeserialize: grpc.deserialize<management_interface_pb.LogFilter>;
    responseSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    responseDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
}
interface IManagementServiceService_ILogListen extends grpc.MethodDefinition<google_protobuf_empty_pb.Empty, management_interface_pb.LogMessage> {
    path: "/mullvad_daemon.management_interface.ManagementService/LogListen";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<google_protobuf_empty_pb.Empty>;
    requestDeserialize: grpc.deserialize<google_protobuf_empty_pb.Empty>;
    responseSerialize: grpc.serialize<management_interface_pb.LogMessage>;
    responseDeserialize: grpc.deserialize<management_interface_pb.LogMessage>;
}

export const ManagementServiceService: IManagementServiceService;

export interface IManagementServiceServer extends grpc.UntypedServiceImplementation {
    connectTunnel: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue>;
    disconnectTunnel: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_wrappers_pb.BoolValue>;
    reconnectTunnel: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue>;
    getTunnelState: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.TunnelState>;
    eventsListen: grpc.handleServerStreamingCall<google_protobuf_empty_pb.Empty, management_interface_pb.DaemonEvent>;
    prepareRestart: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    prepareRestartV2: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    factoryReset: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getCurrentVersion: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue>;
    getVersionInfo: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.AppVersionInfo>;
    isPerformingPostUpgrade: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue>;
    updateRelayLocations: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getRelayLocations: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.RelayList>;
    setRelaySettings: grpc.handleUnaryCall<management_interface_pb.RelaySettings, google_protobuf_empty_pb.Empty>;
    setObfuscationSettings: grpc.handleUnaryCall<management_interface_pb.ObfuscationSettings, google_protobuf_empty_pb.Empty>;
    getSettings: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.Settings>;
    resetSettings: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    setAllowLan: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setShowBetaReleases: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setLockdownMode: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setAutoConnect: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setWireguardMtu: grpc.handleUnaryCall<google_protobuf_wrappers_pb.UInt32Value, google_protobuf_empty_pb.Empty>;
    setWireguardAllowedIps: grpc.handleUnaryCall<management_interface_pb.AllowedIpsList, google_protobuf_empty_pb.Empty>;
    setEnableIpv6: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setQuantumResistantTunnel: grpc.handleUnaryCall<management_interface_pb.QuantumResistantState, google_protobuf_empty_pb.Empty>;
    setEnableDaita: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setDaitaDirectOnly: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    setDaitaSettings: grpc.handleUnaryCall<management_interface_pb.DaitaSettings, google_protobuf_empty_pb.Empty>;
    setDnsOptions: grpc.handleUnaryCall<management_interface_pb.DnsOptions, google_protobuf_empty_pb.Empty>;
    setRelayOverride: grpc.handleUnaryCall<management_interface_pb.RelayOverride, google_protobuf_empty_pb.Empty>;
    clearAllRelayOverrides: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    setEnableRecents: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    createNewAccount: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue>;
    loginAccount: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    logoutAccount: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    getAccountData: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, management_interface_pb.AccountData>;
    getAccountHistory: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.AccountHistory>;
    clearAccountHistory: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getWwwAuthToken: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue>;
    submitVoucher: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, management_interface_pb.VoucherSubmission>;
    getDevice: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.DeviceState>;
    updateDevice: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    listDevices: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, management_interface_pb.DeviceList>;
    removeDevice: grpc.handleUnaryCall<management_interface_pb.DeviceRemoval, google_protobuf_empty_pb.Empty>;
    setWireguardRotationInterval: grpc.handleUnaryCall<google_protobuf_duration_pb.Duration, google_protobuf_empty_pb.Empty>;
    resetWireguardRotationInterval: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    rotateWireguardKey: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getWireguardKey: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.PublicKey>;
    createCustomList: grpc.handleUnaryCall<management_interface_pb.NewCustomList, google_protobuf_wrappers_pb.StringValue>;
    deleteCustomList: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    updateCustomList: grpc.handleUnaryCall<management_interface_pb.CustomList, google_protobuf_empty_pb.Empty>;
    clearCustomLists: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    addApiAccessMethod: grpc.handleUnaryCall<management_interface_pb.NewAccessMethodSetting, management_interface_pb.UUID>;
    removeApiAccessMethod: grpc.handleUnaryCall<management_interface_pb.UUID, google_protobuf_empty_pb.Empty>;
    setApiAccessMethod: grpc.handleUnaryCall<management_interface_pb.UUID, google_protobuf_empty_pb.Empty>;
    updateApiAccessMethod: grpc.handleUnaryCall<management_interface_pb.AccessMethodSetting, google_protobuf_empty_pb.Empty>;
    clearCustomApiAccessMethods: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getCurrentApiAccessMethod: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.AccessMethodSetting>;
    testCustomApiAccessMethod: grpc.handleUnaryCall<management_interface_pb.CustomProxy, google_protobuf_wrappers_pb.BoolValue>;
    testApiAccessMethodById: grpc.handleUnaryCall<management_interface_pb.UUID, google_protobuf_wrappers_pb.BoolValue>;
    getBridges: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.BridgeList>;
    getSplitTunnelProcesses: grpc.handleServerStreamingCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.Int32Value>;
    addSplitTunnelProcess: grpc.handleUnaryCall<google_protobuf_wrappers_pb.Int32Value, google_protobuf_empty_pb.Empty>;
    removeSplitTunnelProcess: grpc.handleUnaryCall<google_protobuf_wrappers_pb.Int32Value, google_protobuf_empty_pb.Empty>;
    clearSplitTunnelProcesses: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    splitTunnelIsSupported: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue>;
    addSplitTunnelApp: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    removeSplitTunnelApp: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    setSplitTunnelState: grpc.handleUnaryCall<google_protobuf_wrappers_pb.BoolValue, google_protobuf_empty_pb.Empty>;
    clearSplitTunnelApps: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    getExcludedProcesses: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.ExcludedProcessList>;
    initPlayPurchase: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.PlayPurchasePaymentToken>;
    verifyPlayPurchase: grpc.handleUnaryCall<management_interface_pb.PlayPurchase, google_protobuf_empty_pb.Empty>;
    needFullDiskPermissions: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.BoolValue>;
    checkVolumes: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    applyJsonSettings: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    exportJsonSettings: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue>;
    getFeatureIndicators: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.FeatureIndicators>;
    disableRelay: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    enableRelay: grpc.handleUnaryCall<google_protobuf_wrappers_pb.StringValue, google_protobuf_empty_pb.Empty>;
    getRolloutThreshold: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.Rollout>;
    regenerateRolloutThreshold: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, management_interface_pb.Rollout>;
    setRolloutThresholdSeed: grpc.handleUnaryCall<management_interface_pb.Seed, google_protobuf_empty_pb.Empty>;
    appUpgrade: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    appUpgradeAbort: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_empty_pb.Empty>;
    appUpgradeEventsListen: grpc.handleServerStreamingCall<google_protobuf_empty_pb.Empty, management_interface_pb.AppUpgradeEvent>;
    getAppUpgradeCacheDir: grpc.handleUnaryCall<google_protobuf_empty_pb.Empty, google_protobuf_wrappers_pb.StringValue>;
    setLogFilter: grpc.handleUnaryCall<management_interface_pb.LogFilter, google_protobuf_empty_pb.Empty>;
    logListen: grpc.handleServerStreamingCall<google_protobuf_empty_pb.Empty, management_interface_pb.LogMessage>;
}

export interface IManagementServiceClient {
    connectTunnel(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    connectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    connectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    reconnectTunnel(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    reconnectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    reconnectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    getTunnelState(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    getTunnelState(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    getTunnelState(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    eventsListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.DaemonEvent>;
    eventsListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.DaemonEvent>;
    prepareRestart(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    prepareRestart(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    prepareRestart(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    factoryReset(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    factoryReset(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    factoryReset(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getCurrentVersion(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getCurrentVersion(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getCurrentVersion(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getVersionInfo(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    getVersionInfo(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    getVersionInfo(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    updateRelayLocations(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getRelayLocations(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    getRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    getRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    setRelaySettings(request: management_interface_pb.RelaySettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRelaySettings(request: management_interface_pb.RelaySettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRelaySettings(request: management_interface_pb.RelaySettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    getSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    getSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    resetSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    resetSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    resetSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaSettings(request: management_interface_pb.DaitaSettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaSettings(request: management_interface_pb.DaitaSettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDaitaSettings(request: management_interface_pb.DaitaSettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDnsOptions(request: management_interface_pb.DnsOptions, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDnsOptions(request: management_interface_pb.DnsOptions, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setDnsOptions(request: management_interface_pb.DnsOptions, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRelayOverride(request: management_interface_pb.RelayOverride, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRelayOverride(request: management_interface_pb.RelayOverride, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRelayOverride(request: management_interface_pb.RelayOverride, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    createNewAccount(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    createNewAccount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    createNewAccount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    loginAccount(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    loginAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    loginAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    logoutAccount(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    logoutAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    logoutAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getAccountData(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    getAccountData(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    getAccountData(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    getAccountHistory(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    getAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    getAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    clearAccountHistory(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getWwwAuthToken(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getWwwAuthToken(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getWwwAuthToken(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    submitVoucher(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    submitVoucher(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    submitVoucher(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    getDevice(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    getDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    getDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    updateDevice(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    listDevices(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    listDevices(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    listDevices(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    removeDevice(request: management_interface_pb.DeviceRemoval, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeDevice(request: management_interface_pb.DeviceRemoval, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeDevice(request: management_interface_pb.DeviceRemoval, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    rotateWireguardKey(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    rotateWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    rotateWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getWireguardKey(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    getWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    getWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    createCustomList(request: management_interface_pb.NewCustomList, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    createCustomList(request: management_interface_pb.NewCustomList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    createCustomList(request: management_interface_pb.NewCustomList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateCustomList(request: management_interface_pb.CustomList, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateCustomList(request: management_interface_pb.CustomList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateCustomList(request: management_interface_pb.CustomList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomLists(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomLists(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomLists(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    removeApiAccessMethod(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setApiAccessMethod(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    testApiAccessMethodById(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    testApiAccessMethodById(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    testApiAccessMethodById(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    getBridges(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    getBridges(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    getBridges(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    getSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<google_protobuf_wrappers_pb.Int32Value>;
    getSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<google_protobuf_wrappers_pb.Int32Value>;
    addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getExcludedProcesses(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    getExcludedProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    getExcludedProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    initPlayPurchase(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    initPlayPurchase(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    initPlayPurchase(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    verifyPlayPurchase(request: management_interface_pb.PlayPurchase, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    verifyPlayPurchase(request: management_interface_pb.PlayPurchase, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    verifyPlayPurchase(request: management_interface_pb.PlayPurchase, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    checkVolumes(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    checkVolumes(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    checkVolumes(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    exportJsonSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    exportJsonSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    exportJsonSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getFeatureIndicators(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    getFeatureIndicators(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    getFeatureIndicators(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    disableRelay(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    disableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    disableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    enableRelay(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    enableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    enableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    getRolloutThreshold(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    getRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    getRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    setRolloutThresholdSeed(request: management_interface_pb.Seed, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRolloutThresholdSeed(request: management_interface_pb.Seed, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setRolloutThresholdSeed(request: management_interface_pb.Seed, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgrade(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgradeAbort(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgradeAbort(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgradeAbort(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    appUpgradeEventsListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.AppUpgradeEvent>;
    appUpgradeEventsListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.AppUpgradeEvent>;
    getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    setLogFilter(request: management_interface_pb.LogFilter, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setLogFilter(request: management_interface_pb.LogFilter, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    setLogFilter(request: management_interface_pb.LogFilter, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    logListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.LogMessage>;
    logListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.LogMessage>;
}

export class ManagementServiceClient extends grpc.Client implements IManagementServiceClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: Partial<grpc.ClientOptions>);
    public connectTunnel(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public connectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public connectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public disconnectTunnel(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public reconnectTunnel(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public reconnectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public reconnectTunnel(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public getTunnelState(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    public getTunnelState(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    public getTunnelState(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.TunnelState) => void): grpc.ClientUnaryCall;
    public eventsListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.DaemonEvent>;
    public eventsListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.DaemonEvent>;
    public prepareRestart(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public prepareRestart(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public prepareRestart(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public prepareRestartV2(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public factoryReset(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public factoryReset(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public factoryReset(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getCurrentVersion(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getCurrentVersion(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getCurrentVersion(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getVersionInfo(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    public getVersionInfo(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    public getVersionInfo(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AppVersionInfo) => void): grpc.ClientUnaryCall;
    public isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public isPerformingPostUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public updateRelayLocations(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getRelayLocations(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    public getRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    public getRelayLocations(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.RelayList) => void): grpc.ClientUnaryCall;
    public setRelaySettings(request: management_interface_pb.RelaySettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRelaySettings(request: management_interface_pb.RelaySettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRelaySettings(request: management_interface_pb.RelaySettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setObfuscationSettings(request: management_interface_pb.ObfuscationSettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    public getSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    public getSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Settings) => void): grpc.ClientUnaryCall;
    public resetSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public resetSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public resetSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAllowLan(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setShowBetaReleases(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setLockdownMode(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setAutoConnect(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardMtu(request: google_protobuf_wrappers_pb.UInt32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardAllowedIps(request: management_interface_pb.AllowedIpsList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableIpv6(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setQuantumResistantTunnel(request: management_interface_pb.QuantumResistantState, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableDaita(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaDirectOnly(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaSettings(request: management_interface_pb.DaitaSettings, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaSettings(request: management_interface_pb.DaitaSettings, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDaitaSettings(request: management_interface_pb.DaitaSettings, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDnsOptions(request: management_interface_pb.DnsOptions, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDnsOptions(request: management_interface_pb.DnsOptions, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setDnsOptions(request: management_interface_pb.DnsOptions, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRelayOverride(request: management_interface_pb.RelayOverride, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRelayOverride(request: management_interface_pb.RelayOverride, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRelayOverride(request: management_interface_pb.RelayOverride, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearAllRelayOverrides(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setEnableRecents(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public createNewAccount(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public createNewAccount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public createNewAccount(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public loginAccount(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public loginAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public loginAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public logoutAccount(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public logoutAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public logoutAccount(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getAccountData(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    public getAccountData(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    public getAccountData(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountData) => void): grpc.ClientUnaryCall;
    public getAccountHistory(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    public getAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    public getAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccountHistory) => void): grpc.ClientUnaryCall;
    public clearAccountHistory(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearAccountHistory(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getWwwAuthToken(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getWwwAuthToken(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getWwwAuthToken(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public submitVoucher(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    public submitVoucher(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    public submitVoucher(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.VoucherSubmission) => void): grpc.ClientUnaryCall;
    public getDevice(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    public getDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    public getDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceState) => void): grpc.ClientUnaryCall;
    public updateDevice(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateDevice(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public listDevices(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    public listDevices(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    public listDevices(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.DeviceList) => void): grpc.ClientUnaryCall;
    public removeDevice(request: management_interface_pb.DeviceRemoval, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeDevice(request: management_interface_pb.DeviceRemoval, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeDevice(request: management_interface_pb.DeviceRemoval, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setWireguardRotationInterval(request: google_protobuf_duration_pb.Duration, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public resetWireguardRotationInterval(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public rotateWireguardKey(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public rotateWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public rotateWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getWireguardKey(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    public getWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    public getWireguardKey(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PublicKey) => void): grpc.ClientUnaryCall;
    public createCustomList(request: management_interface_pb.NewCustomList, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public createCustomList(request: management_interface_pb.NewCustomList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public createCustomList(request: management_interface_pb.NewCustomList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public deleteCustomList(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateCustomList(request: management_interface_pb.CustomList, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateCustomList(request: management_interface_pb.CustomList, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateCustomList(request: management_interface_pb.CustomList, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomLists(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomLists(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomLists(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    public addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    public addApiAccessMethod(request: management_interface_pb.NewAccessMethodSetting, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.UUID) => void): grpc.ClientUnaryCall;
    public removeApiAccessMethod(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setApiAccessMethod(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setApiAccessMethod(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public updateApiAccessMethod(request: management_interface_pb.AccessMethodSetting, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearCustomApiAccessMethods(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    public getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    public getCurrentApiAccessMethod(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.AccessMethodSetting) => void): grpc.ClientUnaryCall;
    public testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public testCustomApiAccessMethod(request: management_interface_pb.CustomProxy, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public testApiAccessMethodById(request: management_interface_pb.UUID, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public testApiAccessMethodById(request: management_interface_pb.UUID, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public testApiAccessMethodById(request: management_interface_pb.UUID, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public getBridges(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    public getBridges(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    public getBridges(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.BridgeList) => void): grpc.ClientUnaryCall;
    public getSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<google_protobuf_wrappers_pb.Int32Value>;
    public getSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<google_protobuf_wrappers_pb.Int32Value>;
    public addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelProcess(request: google_protobuf_wrappers_pb.Int32Value, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public splitTunnelIsSupported(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public addSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public removeSplitTunnelApp(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setSplitTunnelState(request: google_protobuf_wrappers_pb.BoolValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public clearSplitTunnelApps(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getExcludedProcesses(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    public getExcludedProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    public getExcludedProcesses(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.ExcludedProcessList) => void): grpc.ClientUnaryCall;
    public initPlayPurchase(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    public initPlayPurchase(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    public initPlayPurchase(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.PlayPurchasePaymentToken) => void): grpc.ClientUnaryCall;
    public verifyPlayPurchase(request: management_interface_pb.PlayPurchase, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public verifyPlayPurchase(request: management_interface_pb.PlayPurchase, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public verifyPlayPurchase(request: management_interface_pb.PlayPurchase, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public needFullDiskPermissions(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.BoolValue) => void): grpc.ClientUnaryCall;
    public checkVolumes(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public checkVolumes(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public checkVolumes(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public applyJsonSettings(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public exportJsonSettings(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public exportJsonSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public exportJsonSettings(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getFeatureIndicators(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    public getFeatureIndicators(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    public getFeatureIndicators(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.FeatureIndicators) => void): grpc.ClientUnaryCall;
    public disableRelay(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public disableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public disableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public enableRelay(request: google_protobuf_wrappers_pb.StringValue, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public enableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public enableRelay(request: google_protobuf_wrappers_pb.StringValue, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public getRolloutThreshold(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public getRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public getRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public regenerateRolloutThreshold(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: management_interface_pb.Rollout) => void): grpc.ClientUnaryCall;
    public setRolloutThresholdSeed(request: management_interface_pb.Seed, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRolloutThresholdSeed(request: management_interface_pb.Seed, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setRolloutThresholdSeed(request: management_interface_pb.Seed, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgrade(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgrade(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgradeAbort(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgradeAbort(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgradeAbort(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public appUpgradeEventsListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.AppUpgradeEvent>;
    public appUpgradeEventsListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.AppUpgradeEvent>;
    public getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public getAppUpgradeCacheDir(request: google_protobuf_empty_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_wrappers_pb.StringValue) => void): grpc.ClientUnaryCall;
    public setLogFilter(request: management_interface_pb.LogFilter, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setLogFilter(request: management_interface_pb.LogFilter, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public setLogFilter(request: management_interface_pb.LogFilter, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: google_protobuf_empty_pb.Empty) => void): grpc.ClientUnaryCall;
    public logListen(request: google_protobuf_empty_pb.Empty, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.LogMessage>;
    public logListen(request: google_protobuf_empty_pb.Empty, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<management_interface_pb.LogMessage>;
}
