import { IAppVersionInfo } from '../../../shared/daemon-rpc-types';

export interface IUpdateLatestAction {
  type: 'UPDATE_LATEST';
  latestInfo: IAppVersionInfo;
}

export interface IUpdateVersionAction {
  type: 'UPDATE_VERSION';
  version: string;
  consistent: boolean;
  isBeta: boolean;
}

export type VersionAction = IUpdateLatestAction | IUpdateVersionAction;

function updateLatest(latestInfo: IAppVersionInfo): IUpdateLatestAction {
  return {
    type: 'UPDATE_LATEST',
    latestInfo,
  };
}

function updateVersion(
  version: string,
  consistent: boolean,
  isBeta: boolean,
): IUpdateVersionAction {
  return {
    type: 'UPDATE_VERSION',
    version,
    consistent,
    isBeta,
  };
}

export default { updateLatest, updateVersion };
