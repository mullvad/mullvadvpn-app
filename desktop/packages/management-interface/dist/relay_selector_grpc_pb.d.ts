// package: mullvad_daemon.relay_selector
// file: relay_selector.proto

/* tslint:disable */
/* eslint-disable */

import * as grpc from "@grpc/grpc-js";
import * as relay_selector_pb from "./relay_selector_pb";
import * as management_interface_pb from "./management_interface_pb";

interface IRelaySelectorServiceService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    partitionRelays: IRelaySelectorServiceService_IPartitionRelays;
}

interface IRelaySelectorServiceService_IPartitionRelays extends grpc.MethodDefinition<relay_selector_pb.Predicate, relay_selector_pb.RelayPartitions> {
    path: "/mullvad_daemon.relay_selector.RelaySelectorService/PartitionRelays";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<relay_selector_pb.Predicate>;
    requestDeserialize: grpc.deserialize<relay_selector_pb.Predicate>;
    responseSerialize: grpc.serialize<relay_selector_pb.RelayPartitions>;
    responseDeserialize: grpc.deserialize<relay_selector_pb.RelayPartitions>;
}

export const RelaySelectorServiceService: IRelaySelectorServiceService;

export interface IRelaySelectorServiceServer extends grpc.UntypedServiceImplementation {
    partitionRelays: grpc.handleUnaryCall<relay_selector_pb.Predicate, relay_selector_pb.RelayPartitions>;
}

export interface IRelaySelectorServiceClient {
    partitionRelays(request: relay_selector_pb.Predicate, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
    partitionRelays(request: relay_selector_pb.Predicate, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
    partitionRelays(request: relay_selector_pb.Predicate, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
}

export class RelaySelectorServiceClient extends grpc.Client implements IRelaySelectorServiceClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: Partial<grpc.ClientOptions>);
    public partitionRelays(request: relay_selector_pb.Predicate, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
    public partitionRelays(request: relay_selector_pb.Predicate, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
    public partitionRelays(request: relay_selector_pb.Predicate, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: relay_selector_pb.RelayPartitions) => void): grpc.ClientUnaryCall;
}
