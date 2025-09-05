import { type IApplication } from '../../../../../../shared/application-types';

export function applicationGetKey<T extends IApplication>(application: T): string {
  return application.absolutepath;
}
