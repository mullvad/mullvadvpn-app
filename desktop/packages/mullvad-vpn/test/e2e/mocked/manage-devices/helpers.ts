import { IDevice } from '../../../../src/shared/daemon-rpc-types';
import { MockedTestUtils } from '../mocked-utils';

export const createHelpers = (utils: MockedTestUtils) => {
  const setCurrentDevice = async (currentDevice: IDevice) => {
    await utils.ipc.account.device.notify({
      type: 'logged in',
      deviceState: {
        type: 'logged in',
        accountAndDevice: {
          accountNumber: '0000-0000-0000-0000',
          device: {
            id: currentDevice.id,
            name: currentDevice.name,
            created: currentDevice.created,
          },
        },
      },
    });
  };

  const setDevices = async (devices: IDevice[]) => {
    await utils.ipc.account.devices.notify(devices);
  };

  return { setCurrentDevice, setDevices };
};

export type MAnageDevicesHelpers = ReturnType<typeof createHelpers>;
