// @flow

import type { AppVersionInfo } from '../../lib/daemon-rpc';

export type UpdateLatestAction = {
  type: 'UPDATE_LATEST',
  latestInfo: AppVersionInfo,
};

export type UpdateVersionAction = {
  type: 'UPDATE_VERSION',
  version: string,
  consistent: boolean,
};

export type VersionAction = UpdateLatestAction | UpdateVersionAction;

function updateLatest(latestInfo: AppVersionInfo): UpdateLatestAction {
  return {
    type: 'UPDATE_LATEST',
    latestInfo,
  };
}

function updateVersion(version: string, consistent: boolean): UpdateVersionAction {
  return {
    type: 'UPDATE_VERSION',
    version,
    consistent,
  };
}

export default { updateLatest, updateVersion };
