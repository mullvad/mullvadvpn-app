// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('@grpc/grpc-js');
var management_interface_pb = require('./management_interface_pb.js');
var google_protobuf_empty_pb = require('google-protobuf/google/protobuf/empty_pb.js');
var google_protobuf_timestamp_pb = require('google-protobuf/google/protobuf/timestamp_pb.js');
var google_protobuf_wrappers_pb = require('google-protobuf/google/protobuf/wrappers_pb.js');
var google_protobuf_duration_pb = require('google-protobuf/google/protobuf/duration_pb.js');

function serialize_google_protobuf_BoolValue(arg) {
  if (!(arg instanceof google_protobuf_wrappers_pb.BoolValue)) {
    throw new Error('Expected argument of type google.protobuf.BoolValue');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_BoolValue(buffer_arg) {
  return google_protobuf_wrappers_pb.BoolValue.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_google_protobuf_Duration(arg) {
  if (!(arg instanceof google_protobuf_duration_pb.Duration)) {
    throw new Error('Expected argument of type google.protobuf.Duration');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_Duration(buffer_arg) {
  return google_protobuf_duration_pb.Duration.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_google_protobuf_Empty(arg) {
  if (!(arg instanceof google_protobuf_empty_pb.Empty)) {
    throw new Error('Expected argument of type google.protobuf.Empty');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_Empty(buffer_arg) {
  return google_protobuf_empty_pb.Empty.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_google_protobuf_Int32Value(arg) {
  if (!(arg instanceof google_protobuf_wrappers_pb.Int32Value)) {
    throw new Error('Expected argument of type google.protobuf.Int32Value');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_Int32Value(buffer_arg) {
  return google_protobuf_wrappers_pb.Int32Value.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_google_protobuf_StringValue(arg) {
  if (!(arg instanceof google_protobuf_wrappers_pb.StringValue)) {
    throw new Error('Expected argument of type google.protobuf.StringValue');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_StringValue(buffer_arg) {
  return google_protobuf_wrappers_pb.StringValue.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_google_protobuf_UInt32Value(arg) {
  if (!(arg instanceof google_protobuf_wrappers_pb.UInt32Value)) {
    throw new Error('Expected argument of type google.protobuf.UInt32Value');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_google_protobuf_UInt32Value(buffer_arg) {
  return google_protobuf_wrappers_pb.UInt32Value.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AccessMethodSetting(arg) {
  if (!(arg instanceof management_interface_pb.AccessMethodSetting)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AccessMethodSetting');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AccessMethodSetting(buffer_arg) {
  return management_interface_pb.AccessMethodSetting.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AccountData(arg) {
  if (!(arg instanceof management_interface_pb.AccountData)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AccountData');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AccountData(buffer_arg) {
  return management_interface_pb.AccountData.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AccountHistory(arg) {
  if (!(arg instanceof management_interface_pb.AccountHistory)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AccountHistory');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AccountHistory(buffer_arg) {
  return management_interface_pb.AccountHistory.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AllowedIpsList(arg) {
  if (!(arg instanceof management_interface_pb.AllowedIpsList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AllowedIpsList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AllowedIpsList(buffer_arg) {
  return management_interface_pb.AllowedIpsList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AppUpgradeEvent(arg) {
  if (!(arg instanceof management_interface_pb.AppUpgradeEvent)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AppUpgradeEvent');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AppUpgradeEvent(buffer_arg) {
  return management_interface_pb.AppUpgradeEvent.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_AppVersionInfo(arg) {
  if (!(arg instanceof management_interface_pb.AppVersionInfo)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.AppVersionInfo');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_AppVersionInfo(buffer_arg) {
  return management_interface_pb.AppVersionInfo.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_BridgeList(arg) {
  if (!(arg instanceof management_interface_pb.BridgeList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.BridgeList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_BridgeList(buffer_arg) {
  return management_interface_pb.BridgeList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_CustomList(arg) {
  if (!(arg instanceof management_interface_pb.CustomList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.CustomList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_CustomList(buffer_arg) {
  return management_interface_pb.CustomList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_CustomProxy(arg) {
  if (!(arg instanceof management_interface_pb.CustomProxy)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.CustomProxy');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_CustomProxy(buffer_arg) {
  return management_interface_pb.CustomProxy.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DaemonEvent(arg) {
  if (!(arg instanceof management_interface_pb.DaemonEvent)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DaemonEvent');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DaemonEvent(buffer_arg) {
  return management_interface_pb.DaemonEvent.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DaitaSettings(arg) {
  if (!(arg instanceof management_interface_pb.DaitaSettings)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DaitaSettings');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DaitaSettings(buffer_arg) {
  return management_interface_pb.DaitaSettings.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DeviceList(arg) {
  if (!(arg instanceof management_interface_pb.DeviceList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DeviceList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DeviceList(buffer_arg) {
  return management_interface_pb.DeviceList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DeviceRemoval(arg) {
  if (!(arg instanceof management_interface_pb.DeviceRemoval)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DeviceRemoval');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DeviceRemoval(buffer_arg) {
  return management_interface_pb.DeviceRemoval.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DeviceState(arg) {
  if (!(arg instanceof management_interface_pb.DeviceState)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DeviceState');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DeviceState(buffer_arg) {
  return management_interface_pb.DeviceState.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_DnsOptions(arg) {
  if (!(arg instanceof management_interface_pb.DnsOptions)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.DnsOptions');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_DnsOptions(buffer_arg) {
  return management_interface_pb.DnsOptions.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_ExcludedProcessList(arg) {
  if (!(arg instanceof management_interface_pb.ExcludedProcessList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.ExcludedProcessList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_ExcludedProcessList(buffer_arg) {
  return management_interface_pb.ExcludedProcessList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_FeatureIndicators(arg) {
  if (!(arg instanceof management_interface_pb.FeatureIndicators)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.FeatureIndicators');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_FeatureIndicators(buffer_arg) {
  return management_interface_pb.FeatureIndicators.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_LogFilter(arg) {
  if (!(arg instanceof management_interface_pb.LogFilter)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.LogFilter');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_LogFilter(buffer_arg) {
  return management_interface_pb.LogFilter.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_LogMessage(arg) {
  if (!(arg instanceof management_interface_pb.LogMessage)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.LogMessage');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_LogMessage(buffer_arg) {
  return management_interface_pb.LogMessage.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_NewAccessMethodSetting(arg) {
  if (!(arg instanceof management_interface_pb.NewAccessMethodSetting)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.NewAccessMethodSetting');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_NewAccessMethodSetting(buffer_arg) {
  return management_interface_pb.NewAccessMethodSetting.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_NewCustomList(arg) {
  if (!(arg instanceof management_interface_pb.NewCustomList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.NewCustomList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_NewCustomList(buffer_arg) {
  return management_interface_pb.NewCustomList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_ObfuscationSettings(arg) {
  if (!(arg instanceof management_interface_pb.ObfuscationSettings)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.ObfuscationSettings');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_ObfuscationSettings(buffer_arg) {
  return management_interface_pb.ObfuscationSettings.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_PlayPurchase(arg) {
  if (!(arg instanceof management_interface_pb.PlayPurchase)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.PlayPurchase');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_PlayPurchase(buffer_arg) {
  return management_interface_pb.PlayPurchase.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_PlayPurchasePaymentToken(arg) {
  if (!(arg instanceof management_interface_pb.PlayPurchasePaymentToken)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.PlayPurchasePaymentToken');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_PlayPurchasePaymentToken(buffer_arg) {
  return management_interface_pb.PlayPurchasePaymentToken.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_PublicKey(arg) {
  if (!(arg instanceof management_interface_pb.PublicKey)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.PublicKey');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_PublicKey(buffer_arg) {
  return management_interface_pb.PublicKey.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_QuantumResistantState(arg) {
  if (!(arg instanceof management_interface_pb.QuantumResistantState)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.QuantumResistantState');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_QuantumResistantState(buffer_arg) {
  return management_interface_pb.QuantumResistantState.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_RelayList(arg) {
  if (!(arg instanceof management_interface_pb.RelayList)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.RelayList');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_RelayList(buffer_arg) {
  return management_interface_pb.RelayList.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_RelayOverride(arg) {
  if (!(arg instanceof management_interface_pb.RelayOverride)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.RelayOverride');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_RelayOverride(buffer_arg) {
  return management_interface_pb.RelayOverride.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_RelaySettings(arg) {
  if (!(arg instanceof management_interface_pb.RelaySettings)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.RelaySettings');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_RelaySettings(buffer_arg) {
  return management_interface_pb.RelaySettings.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_Rollout(arg) {
  if (!(arg instanceof management_interface_pb.Rollout)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.Rollout');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_Rollout(buffer_arg) {
  return management_interface_pb.Rollout.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_Seed(arg) {
  if (!(arg instanceof management_interface_pb.Seed)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.Seed');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_Seed(buffer_arg) {
  return management_interface_pb.Seed.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_Settings(arg) {
  if (!(arg instanceof management_interface_pb.Settings)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.Settings');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_Settings(buffer_arg) {
  return management_interface_pb.Settings.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_TunnelState(arg) {
  if (!(arg instanceof management_interface_pb.TunnelState)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.TunnelState');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_TunnelState(buffer_arg) {
  return management_interface_pb.TunnelState.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_UUID(arg) {
  if (!(arg instanceof management_interface_pb.UUID)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.UUID');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_UUID(buffer_arg) {
  return management_interface_pb.UUID.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_management_interface_VoucherSubmission(arg) {
  if (!(arg instanceof management_interface_pb.VoucherSubmission)) {
    throw new Error('Expected argument of type mullvad_daemon.management_interface.VoucherSubmission');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_management_interface_VoucherSubmission(buffer_arg) {
  return management_interface_pb.VoucherSubmission.deserializeBinary(new Uint8Array(buffer_arg));
}


var ManagementServiceService = exports.ManagementServiceService = {
  // Control and get tunnel state
connectTunnel: {
    path: '/mullvad_daemon.management_interface.ManagementService/ConnectTunnel',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  disconnectTunnel: {
    path: '/mullvad_daemon.management_interface.ManagementService/DisconnectTunnel',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  reconnectTunnel: {
    path: '/mullvad_daemon.management_interface.ManagementService/ReconnectTunnel',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  getTunnelState: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetTunnelState',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.TunnelState,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_TunnelState,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_TunnelState,
  },
  // Control the daemon and receive events
eventsListen: {
    path: '/mullvad_daemon.management_interface.ManagementService/EventsListen',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.DaemonEvent,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_DaemonEvent,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_DaemonEvent,
  },
  // DEPRECATED: Prefer PrepareRestartV2.
prepareRestart: {
    path: '/mullvad_daemon.management_interface.ManagementService/PrepareRestart',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Takes a a boolean argument which says whether the daemon should stop after
// it is done preparing for a restart.
prepareRestartV2: {
    path: '/mullvad_daemon.management_interface.ManagementService/PrepareRestartV2',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  factoryReset: {
    path: '/mullvad_daemon.management_interface.ManagementService/FactoryReset',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getCurrentVersion: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetCurrentVersion',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  // Get information about the latest available version of the app.
// Note that calling this during an in-app upgrade will cancel the upgrade.
getVersionInfo: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetVersionInfo',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.AppVersionInfo,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_AppVersionInfo,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_AppVersionInfo,
  },
  isPerformingPostUpgrade: {
    path: '/mullvad_daemon.management_interface.ManagementService/IsPerformingPostUpgrade',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  // Relays and tunnel constraints
updateRelayLocations: {
    path: '/mullvad_daemon.management_interface.ManagementService/UpdateRelayLocations',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getRelayLocations: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetRelayLocations',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.RelayList,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_RelayList,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_RelayList,
  },
  setRelaySettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetRelaySettings',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.RelaySettings,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_RelaySettings,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_RelaySettings,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setObfuscationSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetObfuscationSettings',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.ObfuscationSettings,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_ObfuscationSettings,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_ObfuscationSettings,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Settings
getSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetSettings',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.Settings,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_Settings,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_Settings,
  },
  resetSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/ResetSettings',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setAllowLan: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetAllowLan',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setShowBetaReleases: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetShowBetaReleases',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setLockdownMode: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetLockdownMode',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setAutoConnect: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetAutoConnect',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setWireguardMtu: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetWireguardMtu',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.UInt32Value,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_UInt32Value,
    requestDeserialize: deserialize_google_protobuf_UInt32Value,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setWireguardAllowedIps: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetWireguardAllowedIps',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.AllowedIpsList,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_AllowedIpsList,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_AllowedIpsList,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setEnableIpv6: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetEnableIpv6',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setQuantumResistantTunnel: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetQuantumResistantTunnel',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.QuantumResistantState,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_QuantumResistantState,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_QuantumResistantState,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setEnableDaita: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetEnableDaita',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setDaitaDirectOnly: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetDaitaDirectOnly',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setDaitaSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetDaitaSettings',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.DaitaSettings,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_DaitaSettings,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_DaitaSettings,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setDnsOptions: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetDnsOptions',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.DnsOptions,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_DnsOptions,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_DnsOptions,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setRelayOverride: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetRelayOverride',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.RelayOverride,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_RelayOverride,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_RelayOverride,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  clearAllRelayOverrides: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearAllRelayOverrides',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setEnableRecents: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetEnableRecents',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Account management
createNewAccount: {
    path: '/mullvad_daemon.management_interface.ManagementService/CreateNewAccount',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  loginAccount: {
    path: '/mullvad_daemon.management_interface.ManagementService/LoginAccount',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  logoutAccount: {
    path: '/mullvad_daemon.management_interface.ManagementService/LogoutAccount',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getAccountData: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetAccountData',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: management_interface_pb.AccountData,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_mullvad_daemon_management_interface_AccountData,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_AccountData,
  },
  getAccountHistory: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetAccountHistory',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.AccountHistory,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_AccountHistory,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_AccountHistory,
  },
  clearAccountHistory: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearAccountHistory',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getWwwAuthToken: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetWwwAuthToken',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  submitVoucher: {
    path: '/mullvad_daemon.management_interface.ManagementService/SubmitVoucher',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: management_interface_pb.VoucherSubmission,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_mullvad_daemon_management_interface_VoucherSubmission,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_VoucherSubmission,
  },
  // Device management
getDevice: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetDevice',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.DeviceState,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_DeviceState,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_DeviceState,
  },
  updateDevice: {
    path: '/mullvad_daemon.management_interface.ManagementService/UpdateDevice',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  listDevices: {
    path: '/mullvad_daemon.management_interface.ManagementService/ListDevices',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: management_interface_pb.DeviceList,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_mullvad_daemon_management_interface_DeviceList,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_DeviceList,
  },
  removeDevice: {
    path: '/mullvad_daemon.management_interface.ManagementService/RemoveDevice',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.DeviceRemoval,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_DeviceRemoval,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_DeviceRemoval,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // WireGuard key management
setWireguardRotationInterval: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetWireguardRotationInterval',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_duration_pb.Duration,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Duration,
    requestDeserialize: deserialize_google_protobuf_Duration,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  resetWireguardRotationInterval: {
    path: '/mullvad_daemon.management_interface.ManagementService/ResetWireguardRotationInterval',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  rotateWireguardKey: {
    path: '/mullvad_daemon.management_interface.ManagementService/RotateWireguardKey',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getWireguardKey: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetWireguardKey',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.PublicKey,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_PublicKey,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_PublicKey,
  },
  // Custom lists
createCustomList: {
    path: '/mullvad_daemon.management_interface.ManagementService/CreateCustomList',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.NewCustomList,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_mullvad_daemon_management_interface_NewCustomList,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_NewCustomList,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  deleteCustomList: {
    path: '/mullvad_daemon.management_interface.ManagementService/DeleteCustomList',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  updateCustomList: {
    path: '/mullvad_daemon.management_interface.ManagementService/UpdateCustomList',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.CustomList,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_CustomList,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_CustomList,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  clearCustomLists: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearCustomLists',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Access methods
addApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/AddApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.NewAccessMethodSetting,
    responseType: management_interface_pb.UUID,
    requestSerialize: serialize_mullvad_daemon_management_interface_NewAccessMethodSetting,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_NewAccessMethodSetting,
    responseSerialize: serialize_mullvad_daemon_management_interface_UUID,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_UUID,
  },
  removeApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/RemoveApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.UUID,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_UUID,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_UUID,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.UUID,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_UUID,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_UUID,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  updateApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/UpdateApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.AccessMethodSetting,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_AccessMethodSetting,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_AccessMethodSetting,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  clearCustomApiAccessMethods: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearCustomApiAccessMethods',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getCurrentApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetCurrentApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.AccessMethodSetting,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_AccessMethodSetting,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_AccessMethodSetting,
  },
  testCustomApiAccessMethod: {
    path: '/mullvad_daemon.management_interface.ManagementService/TestCustomApiAccessMethod',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.CustomProxy,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_mullvad_daemon_management_interface_CustomProxy,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_CustomProxy,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  testApiAccessMethodById: {
    path: '/mullvad_daemon.management_interface.ManagementService/TestApiAccessMethodById',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.UUID,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_mullvad_daemon_management_interface_UUID,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_UUID,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  // Bridges (Used for reaching the API)
getBridges: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetBridges',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.BridgeList,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_BridgeList,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_BridgeList,
  },
  // Split tunneling (Linux)
getSplitTunnelProcesses: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetSplitTunnelProcesses',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.Int32Value,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Int32Value,
    responseDeserialize: deserialize_google_protobuf_Int32Value,
  },
  addSplitTunnelProcess: {
    path: '/mullvad_daemon.management_interface.ManagementService/AddSplitTunnelProcess',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.Int32Value,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Int32Value,
    requestDeserialize: deserialize_google_protobuf_Int32Value,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  removeSplitTunnelProcess: {
    path: '/mullvad_daemon.management_interface.ManagementService/RemoveSplitTunnelProcess',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.Int32Value,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Int32Value,
    requestDeserialize: deserialize_google_protobuf_Int32Value,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  clearSplitTunnelProcesses: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearSplitTunnelProcesses',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Split tunneling (Linux, Windows)
splitTunnelIsSupported: {
    path: '/mullvad_daemon.management_interface.ManagementService/SplitTunnelIsSupported',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  // Split tunneling (Windows, macOS, Android)
addSplitTunnelApp: {
    path: '/mullvad_daemon.management_interface.ManagementService/AddSplitTunnelApp',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  removeSplitTunnelApp: {
    path: '/mullvad_daemon.management_interface.ManagementService/RemoveSplitTunnelApp',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  setSplitTunnelState: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetSplitTunnelState',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.BoolValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_BoolValue,
    requestDeserialize: deserialize_google_protobuf_BoolValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Split tunneling (Windows, macOS)
clearSplitTunnelApps: {
    path: '/mullvad_daemon.management_interface.ManagementService/ClearSplitTunnelApps',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getExcludedProcesses: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetExcludedProcesses',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.ExcludedProcessList,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_ExcludedProcessList,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_ExcludedProcessList,
  },
  // Play payment (Android)
initPlayPurchase: {
    path: '/mullvad_daemon.management_interface.ManagementService/InitPlayPurchase',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.PlayPurchasePaymentToken,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_PlayPurchasePaymentToken,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_PlayPurchasePaymentToken,
  },
  verifyPlayPurchase: {
    path: '/mullvad_daemon.management_interface.ManagementService/VerifyPlayPurchase',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.PlayPurchase,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_PlayPurchase,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_PlayPurchase,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Check whether the app needs TCC approval for split tunneling (macOS)
needFullDiskPermissions: {
    path: '/mullvad_daemon.management_interface.ManagementService/NeedFullDiskPermissions',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.BoolValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_BoolValue,
    responseDeserialize: deserialize_google_protobuf_BoolValue,
  },
  // Notify the split tunnel monitor that a volume was mounted or dismounted
// (Windows).
checkVolumes: {
    path: '/mullvad_daemon.management_interface.ManagementService/CheckVolumes',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Apply a JSON blob to the settings
// See ../../docs/settings-patch-format.md for a description of the format
applyJsonSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/ApplyJsonSettings',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // Return a JSON blob containing all overridable settings, if there are any
exportJsonSettings: {
    path: '/mullvad_daemon.management_interface.ManagementService/ExportJsonSettings',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  // Get current feature indicators
getFeatureIndicators: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetFeatureIndicators',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.FeatureIndicators,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_FeatureIndicators,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_FeatureIndicators,
  },
  // Debug features
disableRelay: {
    path: '/mullvad_daemon.management_interface.ManagementService/DisableRelay',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  enableRelay: {
    path: '/mullvad_daemon.management_interface.ManagementService/EnableRelay',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_wrappers_pb.StringValue,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_StringValue,
    requestDeserialize: deserialize_google_protobuf_StringValue,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  getRolloutThreshold: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetRolloutThreshold',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.Rollout,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_Rollout,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_Rollout,
  },
  regenerateRolloutThreshold: {
    path: '/mullvad_daemon.management_interface.ManagementService/RegenerateRolloutThreshold',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.Rollout,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_Rollout,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_Rollout,
  },
  setRolloutThresholdSeed: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetRolloutThresholdSeed',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.Seed,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_Seed,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_Seed,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  // App upgrade
appUpgrade: {
    path: '/mullvad_daemon.management_interface.ManagementService/AppUpgrade',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  appUpgradeAbort: {
    path: '/mullvad_daemon.management_interface.ManagementService/AppUpgradeAbort',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  appUpgradeEventsListen: {
    path: '/mullvad_daemon.management_interface.ManagementService/AppUpgradeEventsListen',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.AppUpgradeEvent,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_AppUpgradeEvent,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_AppUpgradeEvent,
  },
  getAppUpgradeCacheDir: {
    path: '/mullvad_daemon.management_interface.ManagementService/GetAppUpgradeCacheDir',
    requestStream: false,
    responseStream: false,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: google_protobuf_wrappers_pb.StringValue,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_google_protobuf_StringValue,
    responseDeserialize: deserialize_google_protobuf_StringValue,
  },
  setLogFilter: {
    path: '/mullvad_daemon.management_interface.ManagementService/SetLogFilter',
    requestStream: false,
    responseStream: false,
    requestType: management_interface_pb.LogFilter,
    responseType: google_protobuf_empty_pb.Empty,
    requestSerialize: serialize_mullvad_daemon_management_interface_LogFilter,
    requestDeserialize: deserialize_mullvad_daemon_management_interface_LogFilter,
    responseSerialize: serialize_google_protobuf_Empty,
    responseDeserialize: deserialize_google_protobuf_Empty,
  },
  logListen: {
    path: '/mullvad_daemon.management_interface.ManagementService/LogListen',
    requestStream: false,
    responseStream: true,
    requestType: google_protobuf_empty_pb.Empty,
    responseType: management_interface_pb.LogMessage,
    requestSerialize: serialize_google_protobuf_Empty,
    requestDeserialize: deserialize_google_protobuf_Empty,
    responseSerialize: serialize_mullvad_daemon_management_interface_LogMessage,
    responseDeserialize: deserialize_mullvad_daemon_management_interface_LogMessage,
  },
};

exports.ManagementServiceClient = grpc.makeGenericClientConstructor(ManagementServiceService, 'ManagementService');
