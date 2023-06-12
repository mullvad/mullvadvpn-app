import { expect } from '@playwright/test';
import { Page } from 'playwright';
import { colors } from '../../../src/config.json';
import { anyOf } from '../utils';

const UNSECURED_COLOR = colors.red;
const SECURE_COLOR = colors.green;
const WHITE_COLOR = colors.white;

const UNSECURE_BUTTON_COLOR = anyOf(colors.red60, colors.red80);
const SECURE_BUTTON_COLOR = anyOf(colors.green, colors.green90);

const getLabel = (page: Page) => page.locator('span[role="status"]');
const getHeader = (page: Page) => page.locator('header');

export async function assertDisconnected(page: Page) {
  await assertTunnelState(page, {
    labelText: 'unsecured connection',
    labelColor: UNSECURED_COLOR,
    headerColor: UNSECURED_COLOR,
    buttonText: 'secure my connection',
    buttonColor: SECURE_BUTTON_COLOR,
  });
}

export async function assertConnecting(page: Page) {
  await assertTunnelState(page, {
    labelText: 'creating secure connection',
    labelColor: WHITE_COLOR,
    headerColor: SECURE_COLOR,
    buttonText: 'cancel',
    buttonColor: UNSECURE_BUTTON_COLOR,
  });
}

export async function assertConnected(page: Page) {
  await assertTunnelState(page, {
    labelText: 'secure connection',
    labelColor: SECURE_COLOR,
    headerColor: SECURE_COLOR,
    buttonText: 'disconnect',
    buttonColor: UNSECURE_BUTTON_COLOR,
  });
}

export async function assertDisconnecting(page: Page) {
  await assertTunnelState(page, {
    headerColor: UNSECURED_COLOR,
    buttonText: 'secure my connection',
    buttonColor: SECURE_BUTTON_COLOR,
  });
}

export async function assertError(page: Page) {
  await assertTunnelState(page, {
    labelText: 'blocked connection',
    labelColor: WHITE_COLOR,
    headerColor: SECURE_COLOR,
  });
}

export async function assertConnectingPq(page: Page) {
  await assertTunnelState(page, {
    labelText: 'creating quantum secure connection',
    labelColor: WHITE_COLOR,
    headerColor: SECURE_COLOR,
    buttonText: 'cancel',
    buttonColor: UNSECURE_BUTTON_COLOR,
  });
}

export async function assertConnectedPq(page: Page) {
  await assertTunnelState(page, {
    labelText: 'quantum secure connection',
    labelColor: SECURE_COLOR,
    headerColor: SECURE_COLOR,
    buttonText: 'disconnect',
    buttonColor: UNSECURE_BUTTON_COLOR,
  });
}

interface TunnelStateContent {
  labelText?: string | RegExp;
  labelColor?: string;
  headerColor: string;
  buttonText?: string;
  buttonColor?: string | RegExp;
}

export async function assertTunnelState(page: Page, content: TunnelStateContent) {
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
    const button = page.locator('button', { hasText: new RegExp(content.buttonText, 'i') });
    await expect(button).toHaveCSS('background-color', content.buttonColor);
  }
}
