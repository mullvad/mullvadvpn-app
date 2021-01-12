import moment from 'moment';
import { ILogInput, ILogOutput, LogLevel } from './logging-types';

export class Logger {
  private outputs: LogOutput[] = [];

  public addOutput(output: LogOutput) {
    this.outputs.push(output);
  }

  public addInput(input: ILogInput) {
    input.on((level: LogLevel, message: string) => this.outputMessage(level, message));
  }

  public log(level: LogLevel, ...data: unknown[]) {
    const time = moment().format('YYYY-MM-DD HH:mm:ss.SSS');
    const message = `[${time}][${LogLevel[level]}] ${data.join(' ')}`;

    this.outputMessage(level, message);
  }

  public error = (...data: unknown[]) => this.log(LogLevel.error, ...data);
  public warn = (...data: unknown[]) => this.log(LogLevel.warning, ...data);
  public info = (...data: unknown[]) => this.log(LogLevel.info, ...data);
  public verbose = (...data: unknown[]) => this.log(LogLevel.verbose, ...data);
  public debug = (...data: unknown[]) => this.log(LogLevel.debug, ...data);

  public dispose() {
    this.outputs.forEach((output) => output.dispose());
  }

  private outputMessage(level: LogLevel, message: string) {
    this.outputs.forEach((output) => output.write(level, message));
  }
}

export abstract class LogOutput implements ILogOutput {
  constructor(private level: LogLevel) {}

  public write(level: LogLevel, message: string) {
    if (level <= this.level) {
      this.writeImpl(level, message);
    }
  }

  public dispose() {
    // no-op unless overridden
  }

  protected abstract writeImpl(level: LogLevel, message: string): void;
}

export class ConsoleOutput extends LogOutput {
  protected writeImpl(level: LogLevel, message: string) {
    switch (level) {
      case LogLevel.error:
        console.error(message);
        break;
      case LogLevel.warning:
        console.warn(message);
        break;
      case LogLevel.info:
        console.info(message);
        break;
      case LogLevel.verbose:
        console.log(message);
        break;
      case LogLevel.debug:
        console.debug(message);
        break;
    }
  }
}

export default new Logger();
