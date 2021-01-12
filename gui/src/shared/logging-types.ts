export type LogLevelNames = 'error' | 'warn' | 'info' | 'verbose' | 'debug';

export interface LogLevel<T extends LogLevelNames = LogLevelNames> {
  name: T;
  level: number;
}

export interface ILogOutput {
  write: (level: LogLevel, message: string) => void;
  dispose: () => void;
}

export interface ILogInput {
  on: (handler: (level: LogLevel, message: string) => void) => void;
}

export const LogLevels: { [K in LogLevelNames]: LogLevel<K> } = {
  error: {
    name: 'error',
    level: 0,
  },
  warn: {
    name: 'warn',
    level: 1,
  },
  info: {
    name: 'info',
    level: 2,
  },
  verbose: {
    name: 'verbose',
    level: 3,
  },
  debug: {
    name: 'debug',
    level: 4,
  },
};
