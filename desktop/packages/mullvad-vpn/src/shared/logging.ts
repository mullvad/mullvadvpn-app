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
    const time = this.getDateString();
    const stringifiedData = data.map(this.stringifyData).join(' ');
    const message = `[${time}][${LogLevel[level]}] ${stringifiedData}`;

    this.outputMessage(level, message);
  }

  public error = (...data: unknown[]) => this.log(LogLevel.error, ...data);
  public warn = (...data: unknown[]) => this.log(LogLevel.warning, ...data);
  public info = (...data: unknown[]) => this.log(LogLevel.info, ...data);
  public verbose = (...data: unknown[]) => this.log(LogLevel.verbose, ...data);
  public debug = (...data: unknown[]) => this.log(LogLevel.debug, ...data);

  public disposeDisposableOutputs() {
    // Keep the outputs that aren't disposable to continue to forward log messages to them.
    this.outputs = this.outputs.filter((output) => {
      output.dispose?.();
      return output.dispose === undefined;
    });
  }

  private getDateString(): string {
    const date = new Date();
    const year = date.getFullYear();
    const month = Number(date.getMonth() + 1)
      .toString()
      .padStart(2, '0');
    const day = Number(date.getDate()).toString().padStart(2, '0');
    const hour = Number(date.getHours()).toString().padStart(2, '0');
    const minute = Number(date.getMinutes()).toString().padStart(2, '0');
    const second = Number(date.getSeconds()).toString().padStart(2, '0');
    const millisecond = Number(date.getMilliseconds()).toString().padStart(3, '0');
    return `${year}-${month}-${day} ${hour}:${minute}:${second}.${millisecond}`;
  }

  private stringifyData(data: unknown): string {
    return typeof data === 'string' ? data : JSON.stringify(data);
  }

  private outputMessage(level: LogLevel, message: string) {
    this.outputs
      .filter((output) => level <= output.level)
      .forEach(async (output) => {
        try {
          await output.write(level, message);
        } catch (e) {
          const error = e as Error;
          console.error(
            `${output.constructor.name}.write: ${error.message}. Original message: ${message}`,
          );
        }
      });
  }
}

export class ConsoleOutput implements ILogOutput {
  private disabled = false;

  constructor(public level: LogLevel) {}

  public write(level: LogLevel, message: string) {
    if (this.disabled) {
      return;
    }

    try {
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
          console.log(message);
          break;
      }
    } catch (error) {
      this.disabled = true;

      const message = error instanceof Object && 'message' in error ? error.message : '';
      logger.error('Disabling console output due to:', message, error);
    }
  }
}

const logger = new Logger();
export default logger;
