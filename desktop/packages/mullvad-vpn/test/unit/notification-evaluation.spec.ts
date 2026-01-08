import { describe, expect, it, vi } from 'vitest';

import NotificationController from '../../src/main/notification-controller';
import { TunnelState } from '../../src/shared/daemon-rpc-types';
import { ErrorStateCause } from '../../src/shared/daemon-rpc-types';
import { FirewallPolicyErrorType } from '../../src/shared/daemon-rpc-types';
import {
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../../src/shared/notifications';

function createController() {
  class TestNotificationController extends NotificationController {
    // @ts-expect-error Way too many methods to mock.
    private createElectronNotification() {
      return {
        show: () => {
          /* no-op */
        },
        close: () => {
          /* no-op */
        },
        on: () => {
          /* no-op */
        },
        removeAllListeners: () => {
          /* no-op */
        },
      };
    }
  }

  return new TestNotificationController({
    openApp: vi.fn(),
    openLink: vi.fn().mockReturnValue(Promise.resolve()),
    openRoute: vi.fn(),
    showNotificationIcon: vi.fn(),
  });
}

describe('System notifications', () => {
  it('should evaluate unspupported version notification to show', () => {
    const controller1 = createController();
    const controller2 = createController();
    const notification = new UnsupportedVersionNotificationProvider({
      supported: false,
      consistent: true,
      suggestedIsBeta: false,
    });

    expect(notification.mayDisplay()).toBe(true);

    const systemNotification = notification.getSystemNotification();
    const result1 = controller1.notify(systemNotification, false, true);
    const result2 = controller2.notify(systemNotification, false, false);

    expect(result1).toBe(true);
    expect(result2).toBe(true);
  });

  it('should evaluate update available notification to show', () => {
    const controller1 = createController();
    const controller2 = createController();
    const notification = new UpdateAvailableNotificationProvider({
      suggestedUpgrade: {
        changelog: [],
        version: '2100.1',
      },
      suggestedIsBeta: false,
    });

    expect(notification.mayDisplay()).toBe(true);

    const systemNotification = notification.getSystemNotification();
    const result1 = controller1.notify(systemNotification, false, true);
    const result2 = controller2.notify(systemNotification, false, false);

    expect(result1).toBe(true);
    expect(result2).toBe(true);
  });

  it('should show unsupported version notification only once', () => {
    const controller = createController();
    const notification = new UnsupportedVersionNotificationProvider({
      supported: false,
      consistent: true,
      suggestedIsBeta: false,
    });

    const systemNotification = notification.getSystemNotification();
    const result1 = controller.notify(systemNotification, false, true);
    const result2 = controller.notify(systemNotification, false, true);

    expect(result1).toBe(true);
    expect(result2).toBe(false);
  });

  it('should not show notification when window is open', () => {
    const controller = createController();
    const notification = new UnsupportedVersionNotificationProvider({
      supported: false,
      consistent: true,
      suggestedIsBeta: false,
    });

    const systemNotification = notification.getSystemNotification();
    const result = controller.notify(systemNotification, true, true);

    expect(result).toBe(false);
  });

  it('Tunnel state notifications should respect notification setting', () => {
    const controller = createController();

    const disconnectedState: TunnelState = { state: 'disconnected', lockedDown: false };
    const connectingState: TunnelState = { state: 'connecting', featureIndicators: undefined };
    const result1 = controller.notifyTunnelState(disconnectedState, false, false, true);
    const result2 = controller.notifyTunnelState(disconnectedState, false, false, false);
    const result3 = controller.notifyTunnelState(connectingState, false, false, true);
    const result4 = controller.notifyTunnelState(connectingState, false, false, false);

    expect(result1).toBe(true);
    expect(result2).toBe(false);
    expect(result3).toBe(true);
    expect(result4).toBe(false);

    const blockingErrorState: TunnelState = {
      state: 'error',
      details: {
        cause: ErrorStateCause.isOffline,
      },
    };
    const result5 = controller.notifyTunnelState(blockingErrorState, false, false, false);
    expect(result5).toBe(false);

    const nonBlockingErrorState: TunnelState = {
      state: 'error',
      details: {
        cause: ErrorStateCause.isOffline,
        blockingError: {
          type: FirewallPolicyErrorType.generic,
        },
      },
    };
    const result6 = controller.notifyTunnelState(nonBlockingErrorState, false, false, false);
    expect(result6).toBe(true);
  });
});
