export type LogLevelNames = 'error' | 'warn' | 'info' | 'verbose' | 'debug';

export interface LogLevel<T extends LogLevelNames = LogLevelNames> {
  name: T;
  level: number;
  consoleFunction: (...data: unknown[]) => void;
}

export const LogLevels: { [K in LogLevelNames]: LogLevel<K> } = {
  error: {
    name: 'error',
    level: 0,
    consoleFunction: console.error,
  },
  warn: {
    name: 'warn',
    level: 1,
    consoleFunction: console.warn,
  },
  info: {
    name: 'info',
    level: 2,
    consoleFunction: console.info,
  },
  verbose: {
    name: 'verbose',
    level: 3,
    consoleFunction: console.log,
  },
  debug: {
    name: 'debug',
    level: 4,
    consoleFunction: console.debug,
  },
};

export type LogFunctions = {
  [key in keyof typeof LogLevels]: (...data: unknown[]) => void;
};

export interface Logger {
  log(level: LogLevel, ...data: unknown[]): void;
}
