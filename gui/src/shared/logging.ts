import moment from 'moment';
import { ILogInput, ILogOutput, LogLevel } from './logging-types';

export class Logger {
  private outputs: ILogOutput[] = [];

  public addOutput(output: ILogOutput) {
    this.outputs.push(output);
  }

  public addInput(input: ILogInput) {
    input.on((level: LogLevel, message: string) => this.outputMessage(level, message));
  }

  public log(level: LogLevel, ...data: unknown[]) {
    const time = moment().format('YYYY-MM-DD HH:mm:ss.SSS');
    const stringifiedData = data.map(this.stringifyData).join(' ');
    const message = `[${time}][${LogLevel[level]}] ${stringifiedData}`;

    this.outputMessage(level, message);
  }

  public error = (...data: unknown[]) => this.log(LogLevel.error, ...data);
  public warn = (...data: unknown[]) => this.log(LogLevel.warning, ...data);
  public info = (...data: unknown[]) => this.log(LogLevel.info, ...data);
  public verbose = (...data: unknown[]) => this.log(LogLevel.verbose, ...data);
  public debug = (...data: unknown[]) => this.log(LogLevel.debug, ...data);

  public dispose() {
    this.outputs.forEach((output) => output.dispose?.());
  }

  private stringifyData(data: unknown): string {
    return typeof data === 'string' ? data : JSON.stringify(data);
  }

  private outputMessage(level: LogLevel, message: string) {
    this.outputs
      .filter((output) => level <= output.level)
      .forEach(async (output) => {
        const maybePromise = output.write(level, message);
        if (maybePromise instanceof Promise) {
          try {
            await maybePromise;
          } catch (e) {
            console.error(`${output.constructor.name}.write: ${e.message}`);
          }
        }
      });
  }
}

export class ConsoleOutput implements ILogOutput {
  constructor(public level: LogLevel) {}

  public write(level: LogLevel, message: string) {
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
