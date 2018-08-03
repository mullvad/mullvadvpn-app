// @flow

export type UpdateVersionAction = {
  type: 'UPDATE_VERSION',
  version: string,
  consistent: boolean,
};

export type VersionAction = UpdateVersionAction;

function updateVersion(version: string, consistent: boolean): UpdateVersionAction {
  return {
    type: 'UPDATE_VERSION',
    version,
    consistent,
  };
}

export default { updateVersion };
