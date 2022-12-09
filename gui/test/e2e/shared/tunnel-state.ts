import { expect } from '@playwright/test';
import { Page } from 'playwright';
import { colors } from '../../../src/config.json';
import { getBackgroundColor, getColor } from '../utils';

const UNSECURED_COLOR = colors.red;
const SECURE_COLOR = colors.green;
const WHITE_COLOR = colors.white;

const getLabel = (page: Page) => page.locator('span[role="status"]');
const getHeader = (page: Page) => page.locator('header');

export async function assertDisconnected(page: Page) {
  const statusLabel = getLabel(page);
  await expect(statusLabel).toContainText(/unsecured connection/i);
  const labelColor = await getColor(statusLabel);
  expect(labelColor).toBe(UNSECURED_COLOR);

  const header = getHeader(page);
  const headerColor = await getBackgroundColor(header);
  expect(headerColor).toBe(UNSECURED_COLOR);

  const button = page.locator('button', { hasText: /secure my connection/i });
  const buttonColor = await getBackgroundColor(button);
  expect(buttonColor).toBe(SECURE_COLOR);
}

export async function assertConnecting(page: Page) {
  const statusLabel = getLabel(page);
  await expect(statusLabel).toContainText(/creating secure connection/i);
  const labelColor = await getColor(statusLabel);
  expect(labelColor).toBe(WHITE_COLOR);

  const header = getHeader(page);
  const headerColor = await getBackgroundColor(header);
  expect(headerColor).toBe(SECURE_COLOR);

  const button = page.locator('button', { hasText: /cancel/i });
  const buttonColor = await getBackgroundColor(button);
  expect(buttonColor).toBe('rgba(227, 64, 57, 0.6)');
}

export async function assertConnected(page: Page) {
  const statusLabel = getLabel(page);
  await expect(statusLabel).toContainText(/secure connection/i);
  const labelColor = await getColor(statusLabel);
  expect(labelColor).toBe(SECURE_COLOR);

  const header = getHeader(page);
  const headerColor = await getBackgroundColor(header);
  expect(headerColor).toBe(SECURE_COLOR);

  const button = page.locator('button', { hasText: /switch location/i });
  const buttonColor = await getBackgroundColor(button);
  expect(buttonColor).toBe('rgba(255, 255, 255, 0.2)');
}

export async function assertDisconnecting(page: Page) {
  const statusLabel = getLabel(page);
  await expect(statusLabel).toBeEmpty();

  const header = getHeader(page);
  const headerColor = await getBackgroundColor(header);
  expect(headerColor).toBe(UNSECURED_COLOR);

  const button = page.locator('button', { hasText: /secure my connection/i });
  const buttonColor = await getBackgroundColor(button);
  expect(buttonColor).toBe(SECURE_COLOR);
}

export async function assertError(page: Page) {
  const statusLabel = getLabel(page);
  await expect(statusLabel).toContainText(/blocked connection/i);
  const labelColor = await getColor(statusLabel);
  expect(labelColor).toBe(WHITE_COLOR);

  const header = getHeader(page);
  const headerColor = await getBackgroundColor(header);
  expect(headerColor).toBe(SECURE_COLOR);
}
