import MainLogger from '../main/logger';
import { Logger, LogLevels } from './logging-types';

export const OLD_LOG_FILES = ['frontend-renderer.log'];

/* eslint-disable @typescript-eslint/no-var-requires */
const ProcessLogger: { new (): Logger } =
  typeof window === 'undefined'
    ? require('../main/logger').default
    : require('../renderer/lib/logger').default;
/* eslint-enable @typescript-eslint/no-var-requires */

export class SharedLogger extends ProcessLogger {
  error = (...data: unknown[]) => this.log(LogLevels.error, ...data);
  warn = (...data: unknown[]) => this.log(LogLevels.warn, ...data);
  info = (...data: unknown[]) => this.log(LogLevels.info, ...data);
  verbose = (...data: unknown[]) => this.log(LogLevels.verbose, ...data);
  debug = (...data: unknown[]) => this.log(LogLevels.debug, ...data);
}

const log = new SharedLogger();
export default log;

export const mainLogger =
  typeof window === 'undefined' ? (log as MainLogger & SharedLogger) : undefined;
