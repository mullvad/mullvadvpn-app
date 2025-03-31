import { DaemonAppUpgradeEvent } from '../shared/daemon-rpc-types';
import { SubscriptionListener } from './daemon-rpc';

let appUpgradeEventListener: SubscriptionListener<DaemonAppUpgradeEvent>;
let appUpgradeAborted = false;

let timeouts: Array<NodeJS.Timeout> = [];

export const triggerMockAppUpgradeAbort = () => {
  appUpgradeAborted = true;
  timeouts.forEach((timeout) => {
    clearTimeout(timeout);
  });
  timeouts = [];
  appUpgradeEventListener.onEvent({
    type: 'APP_UPGRADE_STATUS_ABORTED',
  });
};

export const sendMockAppUpgradeEvent = (event: DaemonAppUpgradeEvent) => {
  if (appUpgradeAborted) {
    return;
  }

  appUpgradeEventListener.onEvent(event);
};

export const triggerMockAppUpgradeEvents = async () => {
  const delay = (ms: number) => {
    if (appUpgradeAborted) {
      return Promise.resolve(null);
    }

    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        resolve(null);
      }, ms);
      timeouts.push(timeout);
    });
  };

  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED',
  });

  await delay(1000);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 0,
    server: 'cdn.mullvad.net',
    timeLeft: 3000,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 10,
    server: 'cdn.mullvad.net',
    timeLeft: 2700,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 20,
    server: 'cdn.mullvad.net',
    timeLeft: 2400,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 30,
    server: 'cdn.mullvad.net',
    timeLeft: 2100,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 40,
    server: 'cdn.mullvad.net',
    timeLeft: 1800,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 50,
    server: 'cdn.mullvad.net',
    timeLeft: 1500,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 60,
    server: 'cdn.mullvad.net',
    timeLeft: 1200,
  });
  await delay(300);
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
    progress: 70,
    server: 'cdn.mullvad.net',
    timeLeft: 900,
  });
  await delay(300);

  // NOTE: COMMENT THIS MOCK EVENT AND UNCOMMENT THE LINES BELOW TO MOCK A SUCCESSFUL DOWNLOAD
  sendMockAppUpgradeEvent({
    type: 'APP_UPGRADE_ERROR',
    error: 'GENERAL_ERROR',
  });

  // NOTE: UNCOMMENT THESE LINES TO MOCK A SUCCESFUL DOWNLOAD
  // sendMockAppUpgradeEvent({
  //   type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
  //   progress: 80,
  //   server: 'cdn.mullvad.net',
  //   timeLeft: 600,
  // });
  // await delay(300);
  // sendMockAppUpgradeEvent({
  //   type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
  //   progress: 90,
  //   server: 'cdn.mullvad.net',
  //   timeLeft: 300,
  // });
  // await delay(300);
  // sendMockAppUpgradeEvent({
  //   type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
  //   progress: 100,
  //   server: 'cdn.mullvad.net',
  //   timeLeft: 0,
  // });
  // await delay(100);
  // sendMockAppUpgradeEvent({
  //   type: 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER',
  // });
  // await delay(500);
  // sendMockAppUpgradeEvent({
  //   type: 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER',
  // });
};

export const mockAppUpgrade = () => {
  appUpgradeAborted = false;
  void triggerMockAppUpgradeEvents();
};

export const mockAppUpgradeAbort = () => {
  triggerMockAppUpgradeAbort();
};

export const mockSubscribeAppUpgradeEventListenerMock = async (
  listener: SubscriptionListener<DaemonAppUpgradeEvent>,
) => {
  appUpgradeEventListener = listener;

  return Promise.resolve();
};
