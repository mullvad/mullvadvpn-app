// GENERATED CODE -- DO NOT EDIT!

// Original file comments:
// Relay Selector service.
//
// This API is used to query which relays are selectable for a set of filters,
// and to provide information about about how to unblock discarded relays.
//
'use strict';
var grpc = require('@grpc/grpc-js');
var relay_selector_pb = require('./relay_selector_pb.js');
var management_interface_pb = require('./management_interface_pb.js');

function serialize_mullvad_daemon_relay_selector_Predicate(arg) {
  if (!(arg instanceof relay_selector_pb.Predicate)) {
    throw new Error('Expected argument of type mullvad_daemon.relay_selector.Predicate');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_relay_selector_Predicate(buffer_arg) {
  return relay_selector_pb.Predicate.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_mullvad_daemon_relay_selector_RelayPartitions(arg) {
  if (!(arg instanceof relay_selector_pb.RelayPartitions)) {
    throw new Error('Expected argument of type mullvad_daemon.relay_selector.RelayPartitions');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_mullvad_daemon_relay_selector_RelayPartitions(buffer_arg) {
  return relay_selector_pb.RelayPartitions.deserializeBinary(new Uint8Array(buffer_arg));
}


var RelaySelectorServiceService = exports.RelaySelectorServiceService = {
  // Partition the available relays by a predicate. The predicate can be
// derived from the current settings or a set of future settings changes
// before they are applied.
//
// Returns a list of matching relays. For each connection using the given
// constraints, one/a pair of these relays will be selected at random.
//
// Also returns a list of non-matching relays along with the set of
// constraints/conditions that made them unavailable.
partitionRelays: {
    path: '/mullvad_daemon.relay_selector.RelaySelectorService/PartitionRelays',
    requestStream: false,
    responseStream: false,
    requestType: relay_selector_pb.Predicate,
    responseType: relay_selector_pb.RelayPartitions,
    requestSerialize: serialize_mullvad_daemon_relay_selector_Predicate,
    requestDeserialize: deserialize_mullvad_daemon_relay_selector_Predicate,
    responseSerialize: serialize_mullvad_daemon_relay_selector_RelayPartitions,
    responseDeserialize: deserialize_mullvad_daemon_relay_selector_RelayPartitions,
  },
};

exports.RelaySelectorServiceClient = grpc.makeGenericClientConstructor(RelaySelectorServiceService, 'RelaySelectorService');
