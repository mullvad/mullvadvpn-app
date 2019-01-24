// @flow

import type { AppVersionInfo } from '../../../main/daemon-rpc';

type UpdateLatestActionPayload = {
  upToDate: boolean,
  nextUpgrade: ?string,
} & AppVersionInfo;

export type UpdateLatestAction = {
  type: 'UPDATE_LATEST',
  latestInfo: UpdateLatestActionPayload,
};

export type UpdateVersionAction = {
  type: 'UPDATE_VERSION',
  version: string,
  consistent: boolean,
};

export type VersionAction = UpdateLatestAction | UpdateVersionAction;

function updateLatest(latestInfo: UpdateLatestActionPayload): UpdateLatestAction {
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
