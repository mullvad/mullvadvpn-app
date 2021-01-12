export enum LogLevel {
  error,
  warning,
  info,
  verbose,
  debug,
}

export interface ILogOutput {
  level: LogLevel;
  write(level: LogLevel, message: string): void;
  dispose?(): void;
}

export interface ILogInput {
  on(handler: (level: LogLevel, message: string) => void): void;
}
