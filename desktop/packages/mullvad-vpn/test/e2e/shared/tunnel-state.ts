import { expect } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../src/renderer/lib/foundations';
import { anyOf } from '../utils';

const DISCONNECTED_COLOR = colors['--color-red'];
const CONNECTED_COLOR = colors['--color-green'];
const WHITE_COLOR = colors['--color-white'];

const DISCONNECTED_BUTTON_COLOR = anyOf(colors['--color-red'], colors['--color-red-80']);
const DISCONNECTING_BUTTON_COLOR = anyOf(colors['--color-green-40']);
const CONNECTED_BUTTON_COLOR = anyOf(colors['--color-green'], colors['--color-green-90']);

const getLabel = (page: Page) => page.locator('span[role="status"]');
const getHeader = (page: Page) => page.locator('header');

export async function expectDisconnected(page: Page) {
  await expectTunnelState(page, {
    labelText: 'DISCONNECTED',
    labelColor: DISCONNECTED_COLOR,
    headerColor: DISCONNECTED_COLOR,
    buttonText: 'Connect',
    buttonColor: CONNECTED_BUTTON_COLOR,
  });
}

export async function expectConnecting(page: Page) {
  await expectTunnelState(page, {
    labelText: 'CONNECTING...',
    labelColor: WHITE_COLOR,
    headerColor: CONNECTED_COLOR,
    buttonText: 'Cancel',
    buttonColor: DISCONNECTED_BUTTON_COLOR,
  });
}

export async function expectConnected(page: Page) {
  await expectTunnelState(page, {
    labelText: 'CONNECTED',
    labelColor: CONNECTED_COLOR,
    headerColor: CONNECTED_COLOR,
    buttonText: 'Disconnect',
    buttonColor: DISCONNECTED_BUTTON_COLOR,
  });
}

export async function expectDisconnecting(page: Page) {
  await expectTunnelState(page, {
    labelText: 'DISCONNECTING...',
    labelColor: WHITE_COLOR,
    headerColor: DISCONNECTED_COLOR,
    buttonText: 'Connect',
    buttonColor: DISCONNECTING_BUTTON_COLOR,
  });
}

export async function expectError(page: Page) {
  await expectTunnelState(page, {
    labelText: 'BLOCKED CONNECTION',
    labelColor: WHITE_COLOR,
    headerColor: CONNECTED_COLOR,
  });
}

interface TunnelStateContent {
  labelText?: string | RegExp;
  labelColor?: string;
  headerColor: string;
  buttonText?: string;
  buttonColor?: string | RegExp;
}

export async function expectTunnelState(page: Page, content: TunnelStateContent) {
  const statusLabel = getLabel(page);
  if (content.labelText && content.labelColor) {
    await expect(statusLabel).toContainText(content.labelText);
    await expect(statusLabel).toHaveCSS('color', content.labelColor);
  } else {
    await expect(statusLabel).toBeEmpty();
  }

  const header = getHeader(page);
  await expect(header).toHaveCSS('background-color', content.headerColor);

  if (content.buttonText && content.buttonColor) {
    const button = page.getByRole('button', { name: content.buttonText }).last();
    await expect(button).toHaveCSS('background-color', content.buttonColor);
  }
}
