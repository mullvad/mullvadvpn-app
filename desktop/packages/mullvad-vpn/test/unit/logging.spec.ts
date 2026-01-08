import fs from 'fs';
import path from 'path';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { backupLogFile, rotateOrDeleteFile } from '../../src/main/logging';
import { Logger } from '../../src/shared/logging';
import { LogLevel } from '../../src/shared/logging-types';

const aPath = path.normalize('log-directory/a.log');
const oldAPath = path.normalize('log-directory/a.old.log');
const bPath = path.normalize('log-directory/b.log');
const oldBPath = path.normalize('log-directory/b.old.log');

const initialFileState = {
  [aPath]: 'a',
  [bPath]: 'b',
  [oldBPath]: 'old b',
};

let files: Record<string, string>;

vi.mock('fs', () => {
  const fsMock = {
    accessSync: (filePath: string) => {
      const normalizedPath = path.normalize(filePath);
      if (files[normalizedPath] === undefined) {
        throw Error('File not found');
      }
    },
    renameSync: (oldPath: string, newPath: string) => {
      const normalizedOldPath = path.normalize(oldPath);
      const normalizedNewPath = path.normalize(newPath);
      files[normalizedNewPath] = files[normalizedOldPath];
      fs.unlinkSync(normalizedOldPath);
    },
    unlinkSync: (filePath: string) => {
      const normalizedPath = path.normalize(filePath);
      delete files[normalizedPath];
    },

    readFileSync: (filePath: string) => {
      const normalizedPath = path.normalize(filePath);
      return files[normalizedPath] ?? '';
    },
  };

  return {
    default: fsMock,
  };
});

describe('Logging', () => {
  beforeEach(() => {
    files = { ...initialFileState };
  });

  it('should backup log file', () => {
    backupLogFile(aPath);
    const oldA = fs.readFileSync(oldAPath).toString();

    expect(fs.accessSync.bind(null, aPath)).to.throw();
    expect(oldA).to.equal(initialFileState[aPath]);
  });

  it('should replace backup file', () => {
    backupLogFile(bPath);
    const oldB = fs.readFileSync(oldBPath).toString();

    expect(fs.accessSync.bind(null, bPath)).to.throw();
    expect(oldB).to.equal(initialFileState[bPath]);
  });

  it('should clean up old log files', () => {
    rotateOrDeleteFile(bPath);
    const oldB = fs.readFileSync(oldBPath).toString();

    expect(fs.accessSync.bind(null, bPath)).to.throw();
    expect(oldB).to.equal(initialFileState[bPath]);

    rotateOrDeleteFile(bPath);

    expect(fs.accessSync.bind(null, bPath)).to.throw();
    expect(fs.accessSync.bind(null, oldBPath)).to.throw();
  });

  it('should only log for the correct log level', () => {
    const logger = new Logger();

    const errorSpy = vi.fn();
    const warningSpy = vi.fn();
    const infoSpy = vi.fn();
    const verboseSpy = vi.fn();
    const debugSpy = vi.fn();

    logger.addOutput({ level: LogLevel.error, write: errorSpy });
    logger.addOutput({ level: LogLevel.warning, write: warningSpy });
    logger.addOutput({ level: LogLevel.info, write: infoSpy });
    logger.addOutput({ level: LogLevel.verbose, write: verboseSpy });
    logger.addOutput({ level: LogLevel.debug, write: debugSpy });

    logger.error();
    logger.warn();
    logger.info();
    logger.verbose();
    logger.debug();

    expect(errorSpy).toHaveBeenCalledTimes(1);
    expect(warningSpy).toHaveBeenCalledTimes(2);
    expect(infoSpy).toHaveBeenCalledTimes(3);
    expect(verboseSpy).toHaveBeenCalledTimes(4);
    expect(debugSpy).toHaveBeenCalledTimes(5);
  });

  it('should format JavaScript types correctly', async () => {
    const logger = new Logger();

    const promise = new Promise((resolve, _reject) => {
      logger.addOutput({
        level: LogLevel.info,
        write: (_, message) => resolve(message.replace(/^.*\[info\] /, '')),
      });
    });

    logger.info('zero', 'one two', 3, [4, 5, 'six', { seven: 'eight' }], {
      nine: 10,
      eleven: [true],
    });
    await expect(promise).resolves.toEqual(
      'zero one two 3 [4,5,"six",{"seven":"eight"}] {"nine":10,"eleven":[true]}',
    );
  });
});
