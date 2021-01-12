import { expect } from 'chai';
import fs from 'fs';
import sinon from 'sinon';
import { it, describe, before, beforeEach, after } from 'mocha';
import { backupLogFile, rotateOrDeleteFile } from '../src/main/logging';

const aPath = 'log-directory/a.log';
const oldAPath = 'log-directory/a.old.log';
const bPath = 'log-directory/b.log';
const oldBPath = 'log-directory/b.old.log';

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

    sandbox.stub(fs, 'accessSync').callsFake((path) => {
      if (files[path as string] === undefined) {
        throw Error('File not found');
      }
    });

    sandbox.stub(fs, 'renameSync').callsFake((oldPath, newPath) => {
      files[newPath as string] = files[oldPath as string];
      fs.unlinkSync(oldPath);
    });

    sandbox.stub(fs, 'unlinkSync').callsFake((path) => {
      delete files[path as string];
    });

    sandbox.stub(fs, 'readFileSync').callsFake((path) => {
      return files[path as string] ?? '';
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
});
