import * as grpc from '@grpc/grpc-js';
import { RelaySelectorServiceClient } from 'management-interface/relay-selector';
import * as grpcTypesRelaySelector from 'management-interface/relay-selector/grpc-types';

import { GrpcClient } from './grpc-client';

export class RelaySelectorRpc extends GrpcClient<RelaySelectorServiceClient> {
  createClient(): RelaySelectorServiceClient {
    return new RelaySelectorServiceClient(
      this.prefixedRpcPath(),
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    );
  }

  public async getMatchingRelays(
    predicate: grpcTypesRelaySelector.Predicate,
  ): Promise<grpcTypesRelaySelector.RelayPartitions> {
    const response = await this.call<
      grpcTypesRelaySelector.Predicate,
      grpcTypesRelaySelector.RelayPartitions
    >(this.client.partitionRelays, predicate);

    return response;
  }
}
