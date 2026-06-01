import * as grpc from '@grpc/grpc-js';
import { RelaySelectorServiceClient } from 'management-interface/relay-selector';

import { GrpcClient } from './grpc-client';

export class RelaySelectorRpc extends GrpcClient<RelaySelectorServiceClient> {
  createClient(): RelaySelectorServiceClient {
    return new RelaySelectorServiceClient(
      this.prefixedRpcPath(),
      grpc.credentials.createInsecure(),
      this.channelOptions(),
    );
  }
}
