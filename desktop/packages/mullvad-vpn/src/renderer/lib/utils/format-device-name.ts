import { capitalizeEveryWord } from '../../../shared/string-helpers';

export function formatDeviceName(deviceName: string) {
  return capitalizeEveryWord(deviceName);
}
