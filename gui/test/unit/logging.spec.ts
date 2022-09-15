import { expect, spy } from 'chai';
import fs from 'fs';
import sinon from 'sinon';
import { it, describe, before, beforeEach, after } from 'mocha';
import path from 'path';
import { Logger } from '../../src/shared/logging';
import { backupLogFile, rotateOrDeleteFile } from '../../src/main/logging';
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

describe('Logging', () => {
  let files: Record<string, string>;
  let sandbox: sinon.SinonSandbox;

  before(() => {
    sandbox = sinon.createSandbox();

    sandbox.stub(fs, 'accessSync').callsFake((filePath) => {
      const normalizedPath = path.normalize(filePath as string);
      if (files[normalizedPath] === undefined) {
        throw Error('File not found');
      }
    });

    sandbox.stub(fs, 'renameSync').callsFake((oldPath, newPath) => {
      const normalizedOldPath = path.normalize(oldPath as string);
      const normalizedNewPath = path.normalize(newPath as string);
      files[normalizedNewPath] = files[normalizedOldPath];
      fs.unlinkSync(normalizedOldPath);
    });

    sandbox.stub(fs, 'unlinkSync').callsFake((filePath) => {
      const normalizedPath = path.normalize(filePath as string);
      delete files[normalizedPath];
    });

    sandbox.stub(fs, 'readFileSync').callsFake((filePath) => {
      const normalizedPath = path.normalize(filePath as string);
      return files[normalizedPath] ?? '';
    });
  });

  after(() => {
    sandbox.restore();
  });

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

    const errorSpy = spy();
    const warningSpy = spy();
    const infoSpy = spy();
    const verboseSpy = spy();
    const debugSpy = spy();

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

    expect(errorSpy).to.have.been.called.exactly(1);
    expect(warningSpy).to.have.been.called.exactly(2);
    expect(infoSpy).to.have.been.called.exactly(3);
    expect(verboseSpy).to.have.been.called.exactly(4);
    expect(debugSpy).to.have.been.called.exactly(5);
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
    await expect(promise).to.eventually.be.fulfilled.then((result) => {
      expect(result).to.equal(
        'zero one two 3 [4,5,"six",{"seven":"eight"}] {"nine":10,"eleven":[true]}',
      );
    });
  });
});
