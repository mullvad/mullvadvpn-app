import { Page } from 'playwright';

import {
  FeatureIndicator,
  ILocation,
  ITunnelEndpoint,
} from '../../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils } from '../mocked-utils';

const endpoint: ITunnelEndpoint = {
  address: 'wg10:80',
  protocol: 'tcp',
  quantumResistant: false,
  daita: false,
};

const mockDisconnectedLocation: ILocation = {
  country: 'Sweden',
  city: 'Gothenburg',
  latitude: 58,
  longitude: 12,
  mullvadExitIp: false,
};

const mockConnectedLocation: ILocation = { ...mockDisconnectedLocation, mullvadExitIp: true };

export const createHelpers = ({
  utils,
}: {
  page: Page;
  routes: RoutesObjectModel;
  utils: MockedTestUtils;
}) => {
  const connectWithFeatures = async (featureIndicators: FeatureIndicator[] | undefined) => {
    await utils.ipc.tunnel[''].notify({
      state: 'connected',
      details: { endpoint, location: mockConnectedLocation },
      featureIndicators: featureIndicators,
    });
  };

  const disconnect = () =>
    utils.ipc.tunnel[''].notify({
      state: 'disconnected',
      lockedDown: false,
    });

  return { connectWithFeatures, disconnect };
};

export type FeatureIndicatorsHelpers = ReturnType<typeof createHelpers>;
