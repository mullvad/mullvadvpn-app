// @flow

import { expect } from 'chai';
import { check, failFast } from './ipc-helpers';

export class IpcChain {
  _expectedCalls: Array<string>;
  _recordedCalls: Array<string>;
  _mockIpc: {};
  _done: (?Error) => void;
  _aborted: boolean;

  constructor(mockIpc: {}) {
    this._expectedCalls = [];
    this._recordedCalls = [];
    this._mockIpc = mockIpc;
    this._aborted = false;
  }

  require<R>(ipcCall: string): StepBuilder<R> {
    const builder = new StepBuilder(ipcCall);
    this._expectedCalls.push(ipcCall);
    this._addStep(builder);

    return builder;
  }

  _addStep<R>(step: StepBuilder<R>) {
    this._mockIpc[step.ipcCall] = (...args: Array<mixed>) => {
      return new Promise((r) => this._stepPromiseCallback(step, r, args));
    };
  }

  _stepPromiseCallback<R>(step: StepBuilder<R>, resolve: (?R) => void, args: Array<mixed>) {
    if (this._aborted) {
      return;
    }

    this._registerCall(step.ipcCall);

    const inputValidation = step.inputValidation;
    if (inputValidation) {
      const failedInputValidation = failFast(() => {
        inputValidation(...args);
      }, this._done);

      if (failedInputValidation) {
        this._abort();
        return;
      }
    }

    if (this._isLastCall()) {
      this._ensureChainCalledCorrectly();
    }

    resolve(step.returnValue);
  }

  _abort() {
    this._aborted = true;
  }

  _isLastCall(): boolean {
    return this._recordedCalls.length === this._expectedCalls.length;
  }

  _ensureChainCalledCorrectly() {
    check(() => {
      expect(this._expectedCalls).to.deep.equal(this._recordedCalls);
    }, this._done);
  }

  _registerCall(ipcCall: string) {
    this._recordedCalls.push(ipcCall);
  }

  onSuccessOrFailure(done: (*) => void) {
    this._done = done;
  }
}

class StepBuilder<R> {
  ipcCall: string;
  inputValidation: ?(...args: Array<mixed>) => void;
  returnValue: ?R;

  constructor(ipcCall: string) {
    this.ipcCall = ipcCall;
  }

  withInputValidation(iv: (...args: Array<mixed>) => void): this {
    this.inputValidation = iv;
    return this;
  }

  withReturnValue(rv: R): this {
    this.returnValue = rv;
    return this;
  }
}
