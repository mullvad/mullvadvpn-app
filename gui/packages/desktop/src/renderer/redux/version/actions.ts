import { IAppVersionInfo } from '../../../shared/daemon-rpc-types';

interface IUpdateLatestActionPayload extends IAppVersionInfo {
  upToDate: boolean;
  nextUpgrade?: string;
}

export interface IUpdateLatestAction {
  type: 'UPDATE_LATEST';
  latestInfo: IUpdateLatestActionPayload;
}

export interface IUpdateVersionAction {
  type: 'UPDATE_VERSION';
  version: string;
  consistent: boolean;
}

export type VersionAction = IUpdateLatestAction | IUpdateVersionAction;

function updateLatest(latestInfo: IUpdateLatestActionPayload): IUpdateLatestAction {
  return {
    type: 'UPDATE_LATEST',
    latestInfo,
  };
}

function updateVersion(version: string, consistent: boolean): IUpdateVersionAction {
  return {
    type: 'UPDATE_VERSION',
    version,
    consistent,
  };
}

export default { updateLatest, updateVersion };
