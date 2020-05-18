import { IAppVersionInfo } from '../../../shared/daemon-rpc-types';

interface IUpdateLatestActionPayload extends IAppVersionInfo {
  nextUpgrade: string | null;
}

export interface IUpdateLatestAction {
  type: 'UPDATE_LATEST';
  latestInfo: IUpdateLatestActionPayload;
}

export interface IUpdateVersionAction {
  type: 'UPDATE_VERSION';
  version: string;
  consistent: boolean;
  currentIsBeta: boolean;
}

export type VersionAction = IUpdateLatestAction | IUpdateVersionAction;

function updateLatest(latestInfo: IUpdateLatestActionPayload): IUpdateLatestAction {
  return {
    type: 'UPDATE_LATEST',
    latestInfo,
  };
}

function updateVersion(
  version: string,
  consistent: boolean,
  currentIsBeta: boolean,
): IUpdateVersionAction {
  return {
    type: 'UPDATE_VERSION',
    version,
    consistent,
    currentIsBeta,
  };
}

export default { updateLatest, updateVersion };
