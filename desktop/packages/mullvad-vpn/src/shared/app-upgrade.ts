import { DaemonAppUpgradeError, DaemonAppUpgradeEventStatus } from './daemon-rpc-types';

export type AppUpgradeEvent = DaemonAppUpgradeEventStatus;

export type AppUpgradeError = DaemonAppUpgradeError | 'START_INSTALLER_FAILED';
