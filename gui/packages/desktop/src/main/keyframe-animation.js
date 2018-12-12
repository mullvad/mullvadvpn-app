// @flow

export type OnFrameFn = (frame: number) => void;
export type OnFinishFn = (void) => void;
export type KeyframeAnimationOptions = {
  start?: number,
  end: number,
};
export type KeyframeAnimationRange = [number, number];

export default class KeyframeAnimation {
  _speed: number = 200; // ms

  _onFrame: ?OnFrameFn;
  _onFinish: ?OnFinishFn;

  _currentFrame: number = 0;
  _targetFrame: number = 0;

  _isRunning: boolean = false;
  _isFinished: boolean = false;

  _timeout = null;

  set onFrame(newValue: ?OnFrameFn) {
    this._onFrame = newValue;
  }
  get onFrame(): ?OnFrameFn {
    return this._onFrame;
  }

  // called when animation finished
  set onFinish(newValue: ?OnFinishFn) {
    this._onFinish = newValue;
  }
  get onFinish(): ?OnFinishFn {
    return this._onFinish;
  }

  // pace per frame in ms
  set speed(newValue: number) {
    this._speed = parseInt(newValue);
  }
  get speed(): number {
    return this._speed;
  }

  get isFinished(): boolean {
    return this._isFinished;
  }

  play(options: KeyframeAnimationOptions) {
    const { start, end } = options;

    if (start !== undefined) {
      this._currentFrame = start;
    }

    this._targetFrame = end;

    this._isRunning = true;
    this._isFinished = false;

    this._unscheduleUpdate();

    this._render();
    this._scheduleUpdate();
  }

  stop() {
    this._isRunning = false;
    this._unscheduleUpdate();
  }

  _unscheduleUpdate() {
    if (this._timeout) {
      clearTimeout(this._timeout);
      this._timeout = null;
    }
  }

  _scheduleUpdate() {
    this._timeout = setTimeout(() => this._onUpdateFrame(), this._speed);
  }

  _render() {
    if (this._onFrame) {
      this._onFrame(this._currentFrame);
    }
  }

  _didFinish() {
    this._isFinished = true;
    this._isRunning = false;

    if (this._onFinish) {
      this._onFinish();
    }
  }

  _onUpdateFrame() {
    this._advanceFrame();

    if (!this._isFinished) {
      this._render();

      // check once again since onFrame() may stop animation
      if (this._isRunning) {
        this._scheduleUpdate();
      }
    }
  }

  _advanceFrame() {
    if (this._isFinished) {
      return;
    }

    if (this._currentFrame === this._targetFrame) {
      this._didFinish();
    } else if (this._currentFrame < this._targetFrame) {
      this._currentFrame += 1;
    } else {
      this._currentFrame -= 1;
    }
  }
}
