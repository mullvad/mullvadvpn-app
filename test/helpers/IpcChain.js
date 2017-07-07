// @flow

import { expect } from 'chai';
import { check, failFast } from './ipc-helpers';

export class IpcChain {
  _expectedCalls: Array<string>;
  _recordedCalls: Array<string>;
  _mockIpc: {};
  _done: (*) => void;

  constructor(mockIpc: {}) {
    this._expectedCalls = [];
    this._recordedCalls = [];
    this._mockIpc = mockIpc;
  }

  require(ipcCall: string): StepBuilder {
    this._expectedCalls.push(ipcCall);
    return new StepBuilder(ipcCall, this._addStep.bind(this));
  }

  _addStep(step: StepBuilder) {
    const me = this;
    this._mockIpc[step.ipcCall] = function() {
      return new Promise(r => me._stepPromiseCallback(step, r, arguments));
    };
  }

  _stepPromiseCallback(step, resolve, args) {
    this._registerCall(step.ipcCall);

    if (step.inputValidation) {
      failFast(() => step.inputValidation(...args), this._done);
    }

    if (this._isLastCall()) {
      this._ensureChainCalledCorrectly();
    }

    resolve(step.returnValue);
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

class StepBuilder {
  ipcCall: string;
  inputValidation: () => void;
  returnValue: *;
  _cb: (StepBuilder) => void;

  constructor(ipcCall: string, cb: (StepBuilder)=> void) {
    this.ipcCall = ipcCall;
    this._cb = cb;
  }

  withInputValidation(iv: () => void): StepBuilder {
    this.inputValidation = iv;
    return this;
  }

  withReturnValue(rv: *): StepBuilder {
    this.returnValue = rv;
    return this;
  }

  done() {
    this._cb(this);
  }
}
