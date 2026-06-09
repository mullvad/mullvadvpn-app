import * as grpc from '@grpc/grpc-js';
import { RelaySelectorServiceClient } from 'management-interface/relay-selector';
import * as grpcTypesRelaySelector from 'management-interface/relay-selector/grpc-types';

import {
  RelaySelectorPartitions,
  type RelaySelectorPredicate,
} from '../shared/relay-selector-rpc-types';
import { GrpcClient } from './grpc-client';
import {
  convertFromRelaySelectorPartitions,
  convertToRelaySelectorPredicate,
} from './relay-selector-grpc-type-conversions';

export class RelaySelectorRpc extends GrpcClient<RelaySelectorServiceClient> {
  createClient(): RelaySelectorServiceClient {
    return new RelaySelectorServiceClient(
      this.prefixedRpcPath(),
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    );
  }

  public async getRelayPartitions(
    predicate: RelaySelectorPredicate,
  ): Promise<RelaySelectorPartitions> {
    const grpcPredicate = convertToRelaySelectorPredicate(predicate);

    const response = await this.call<
      grpcTypesRelaySelector.Predicate,
      grpcTypesRelaySelector.RelayPartitions
    >(this.client.partitionRelays, grpcPredicate);

    return convertFromRelaySelectorPartitions(response);
  }
}
