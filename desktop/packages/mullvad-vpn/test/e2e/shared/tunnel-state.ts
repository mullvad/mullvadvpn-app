import { expect } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../src/config.json';
import { anyOf } from '../utils';

const DISCONNECTED_COLOR = colors.red;
const CONNECTED_COLOR = colors.green;
const WHITE_COLOR = colors.white;

const DISCONNECTED_BUTTON_COLOR = anyOf(colors.red, colors.red80);
const DISCONNECTING_BUTTON_COLOR = anyOf(colors.green40);
const CONNECTED_BUTTON_COLOR = anyOf(colors.green, colors.green90);

const getLabel = (page: Page) => page.locator('span[role="status"]');
const getHeader = (page: Page) => page.locator('header');

export async function expectDisconnected(page: Page) {
  await expectTunnelState(page, {
    labelText: 'disconnected',
    labelColor: DISCONNECTED_COLOR,
    headerColor: DISCONNECTED_COLOR,
    buttonText: 'connect',
    buttonColor: CONNECTED_BUTTON_COLOR,
  });
}

export async function expectConnecting(page: Page) {
  await expectTunnelState(page, {
    labelText: 'connecting',
    labelColor: WHITE_COLOR,
    headerColor: CONNECTED_COLOR,
    buttonText: 'cancel',
    buttonColor: DISCONNECTED_BUTTON_COLOR,
  });
}

export async function expectConnected(page: Page) {
  await expectTunnelState(page, {
    labelText: 'connected',
    labelColor: CONNECTED_COLOR,
    headerColor: CONNECTED_COLOR,
    buttonText: 'disconnect',
    buttonColor: DISCONNECTED_BUTTON_COLOR,
  });
}

export async function expectDisconnecting(page: Page) {
  await expectTunnelState(page, {
    labelText: 'disconnecting',
    labelColor: WHITE_COLOR,
    headerColor: DISCONNECTED_COLOR,
    buttonText: 'connect',
    buttonColor: DISCONNECTING_BUTTON_COLOR,
  });
}

export async function expectError(page: Page) {
  await expectTunnelState(page, {
    labelText: 'blocked connection',
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
    await expect(statusLabel).toContainText(new RegExp(content.labelText, 'i'));
    await expect(statusLabel).toHaveCSS('color', content.labelColor);
  } else {
    await expect(statusLabel).toBeEmpty();
  }

  const header = getHeader(page);
  await expect(header).toHaveCSS('background-color', content.headerColor);

  if (content.buttonText && content.buttonColor) {
    const button = page.locator('button', { hasText: new RegExp(`^${content.buttonText}$`, 'i') });
    await expect(button).toHaveCSS('background-color', content.buttonColor);
  }
}
