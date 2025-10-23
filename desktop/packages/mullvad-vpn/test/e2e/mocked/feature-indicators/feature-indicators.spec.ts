import { expect, Locator, test } from '@playwright/test';
import { Page } from 'playwright';

import { FeatureIndicator } from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, FeatureIndicatorsHelpers } from './helpers';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;
let helpers: FeatureIndicatorsHelpers;

type FeatureIndicatorTestOption = {
  testId: string;
  featureIndicator: FeatureIndicator;
  featureIndicatorLabel: string;
  route: RoutePath;
  skip?: boolean;
};

type FeatureIndicatorWithOptionTestOption = FeatureIndicatorTestOption & {
  option: {
    name?: string;
    type?: 'switch' | 'listbox' | 'accordion' | 'input';
  };
};

const featureIndicatorWithoutOption: FeatureIndicatorTestOption[] = [
  {
    testId: 'DAITA multihop',
    featureIndicator: FeatureIndicator.daitaMultihop,
    route: RoutePath.daitaSettings,
    featureIndicatorLabel: 'DAITA: Multihop',
  },
  {
    testId: 'split tunneling',
    featureIndicator: FeatureIndicator.splitTunneling,
    route: RoutePath.splitTunneling,
    featureIndicatorLabel: 'Split tunneling',
    skip: process.platform === 'linux',
  },
  {
    testId: 'server ip override',
    featureIndicator: FeatureIndicator.serverIpOverride,
    route: RoutePath.settingsImport,
    featureIndicatorLabel: 'Server ip override',
  },
];

const featureIndicatorWithOption: FeatureIndicatorWithOptionTestOption[] = [
  {
    testId: 'DAITA',
    featureIndicator: FeatureIndicator.daita,
    route: RoutePath.daitaSettings,
    featureIndicatorLabel: 'DAITA',
    option: { name: 'Enable', type: 'switch' },
  },
  {
    testId: 'UDP over TCP',
    featureIndicator: FeatureIndicator.udp2tcp,
    route: RoutePath.censorshipCircumvention,
    featureIndicatorLabel: 'Obfuscation',
    option: { name: 'Obfuscation', type: 'listbox' },
  },
  {
    testId: 'shadowsocks',
    featureIndicator: FeatureIndicator.shadowsocks,
    route: RoutePath.censorshipCircumvention,
    featureIndicatorLabel: 'Obfuscation',
    option: { name: 'Obfuscation', type: 'listbox' },
  },
  {
    testId: 'QUIC',
    featureIndicator: FeatureIndicator.quic,
    route: RoutePath.censorshipCircumvention,
    featureIndicatorLabel: 'Obfuscation',
    option: { name: 'Obfuscation', type: 'listbox' },
  },
  {
    testId: 'LWO',
    featureIndicator: FeatureIndicator.lwo,
    route: RoutePath.censorshipCircumvention,
    featureIndicatorLabel: 'Obfuscation',
    option: { name: 'Obfuscation', type: 'listbox' },
  },
  {
    testId: 'multihop',
    featureIndicator: FeatureIndicator.multihop,
    route: RoutePath.multihopSettings,
    featureIndicatorLabel: 'Multihop',
    option: { name: 'Enable', type: 'switch' },
  },
  {
    testId: 'custom dns',
    featureIndicator: FeatureIndicator.customDns,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'Custom DNS',
    option: { name: 'Use custom DNS server', type: 'switch' },
  },
  {
    testId: 'MTU',
    featureIndicator: FeatureIndicator.customMtu,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'MTU',
    option: { name: 'MTU', type: 'input' },
  },
  {
    testId: 'local network sharing',
    featureIndicator: FeatureIndicator.lanSharing,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'Local network sharing',
    option: { name: 'Local network sharing', type: 'switch' },
  },
  {
    testId: 'lockdown mode',
    featureIndicator: FeatureIndicator.lockdownMode,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'Lockdown mode',
    option: { name: 'Lockdown mode', type: 'switch' },
  },
  {
    testId: 'quantum resistance',
    featureIndicator: FeatureIndicator.quantumResistance,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'Quantum resistance',
    option: { name: 'Quantum-resistant tunnel', type: 'switch' },
  },
  {
    testId: 'dns content blockers',
    featureIndicator: FeatureIndicator.dnsContentBlockers,
    route: RoutePath.vpnSettings,
    featureIndicatorLabel: 'DNS content blockers',
    option: { name: 'DNS content blockers', type: 'accordion' },
  },
];

test.describe('Feature indicators', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    helpers = createHelpers({ page, routes, utils: util });

    await util.expectRoute(RoutePath.main);
  });

  test.beforeEach(async () => {
    await helpers.disconnect();
    await routes.wireguardSettings.goBackToRoute(RoutePath.main);
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  async function expectFeatureIndicators(expectedIndicators: Array<string>, only = true) {
    const indicators = routes.main.selectors.featureIndicators();
    if (only) {
      await expect(indicators).toHaveCount(expectedIndicators.length);
    }

    for (const indicator of expectedIndicators) {
      await expect(routes.main.selectors.featureIndicator(indicator)).toBeVisible();
    }
  }

  test('Should show no feature indicators when disconnected', async () => {
    await expectFeatureIndicators([]);
    await helpers.connectWithFeatures(undefined);

    await expectFeatureIndicators([]);

    const ellipsis = routes.main.selectors.moreFeatureIndicator();
    await expect(ellipsis).not.toBeVisible();

    await page.getByTestId('connection-panel-chevron').click();
    await expect(ellipsis).not.toBeVisible();

    await expectFeatureIndicators([]);
    await page.getByTestId('connection-panel-chevron').click();
  });

  test('Should show no feature indicators when connected with no active features', async () => {
    await helpers.connectWithFeatures(undefined);
    await expectFeatureIndicators([]);

    const ellipsis = routes.main.selectors.moreFeatureIndicator();
    await expect(ellipsis).not.toBeVisible();
  });

  test('Should show feature indicators when connected with active features', async () => {
    await helpers.connectWithFeatures([FeatureIndicator.daita, FeatureIndicator.quantumResistance]);
    await expectFeatureIndicators(['DAITA', 'Quantum Resistance']);
  });

  test('Should show a subset of feature indicators when connected with many active features', async () => {
    await helpers.connectWithFeatures([
      FeatureIndicator.daita,
      FeatureIndicator.udp2tcp,
      FeatureIndicator.customMtu,
      FeatureIndicator.lanSharing,
      FeatureIndicator.serverIpOverride,
      FeatureIndicator.customDns,
      FeatureIndicator.lockdownMode,
      FeatureIndicator.quantumResistance,
      FeatureIndicator.multihop,
    ]);

    const ellipsis = routes.main.selectors.moreFeatureIndicator();
    await expect(ellipsis).toBeVisible();

    await ellipsis.click();

    await expectFeatureIndicators([
      'DAITA',
      'Quantum resistance',
      'MTU',
      'Obfuscation',
      'Local network sharing',
      'Lockdown mode',
      'Multihop',
      'Custom DNS',
      'Server IP override',
    ]);
  });

  const clickFeatureIndicator = async (featureIndicatorLabel: string, route: RoutePath) => {
    const indicator = routes.main.selectors.featureIndicator(featureIndicatorLabel);
    await expect(indicator).toBeVisible();
    await indicator.click();
    await util.expectRoute(route);
  };

  featureIndicatorWithoutOption.forEach(
    ({ testId, featureIndicator, route, featureIndicatorLabel, skip }) => {
      test(`Should navigate to setting when clicking on ${testId} feature indicator`, async () => {
        if (skip === true) {
          test.skip();
        }

        await helpers.connectWithFeatures([featureIndicator]);
        await clickFeatureIndicator(featureIndicatorLabel, route);

        await util.expectRoute(route);
      });
    },
  );

  featureIndicatorWithOption.forEach(
    ({ testId, featureIndicator, route, featureIndicatorLabel, option }) => {
      test(`Should navigate to setting when clicking on ${testId} feature indicator`, async () => {
        await helpers.connectWithFeatures([featureIndicator]);
        await clickFeatureIndicator(featureIndicatorLabel, route);

        const { name, type } = option;
        let element: Locator | undefined = undefined;
        if (type === 'accordion') {
          element = page.getByRole('button', { name });
        } else if (type === 'listbox') {
          element = page.getByRole('listbox', { name });
        } else if (type === 'input') {
          element = page.getByRole('textbox', { name });
        } else {
          element = page.getByRole('switch', { name });
        }
        await expect(element).toBeInViewport();

        await util.expectRoute(route);
      });
    },
  );
});
