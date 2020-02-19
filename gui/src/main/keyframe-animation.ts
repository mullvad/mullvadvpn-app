export type OnFrameFn = (frame: number) => void;
export type OnFinishFn = () => void;

export interface IKeyframeAnimationOptions {
  start?: number;
  end: number;
}
export type KeyframeAnimationRange = [number, number];

export default class KeyframeAnimation {
  private speedValue = 200; // ms

  private onFrameValue?: OnFrameFn;
  private onFinishValue?: OnFinishFn;

  private currentFrameValue = 0;
  private targetFrame = 0;

  private isRunningValue = false;
  private isFinishedValue = false;

  private timeout?: NodeJS.Timeout;

  get currentFrame(): number {
    return this.currentFrameValue;
  }

  // This setter is only meant to be used when running tests
  // @internal
  set currentFrame(newValue: number) {
    if (process.env.NODE_ENV === 'test') {
      this.currentFrameValue = newValue;
    } else {
      throw new Error('The setter for currentFrame is only available in test environment.');
    }
  }

  set onFrame(newValue: OnFrameFn | undefined) {
    this.onFrameValue = newValue;
  }
  get onFrame(): OnFrameFn | undefined {
    return this.onFrameValue;
  }

  // called when animation finished
  set onFinish(newValue: OnFinishFn | undefined) {
    this.onFinishValue = newValue;
  }
  get onFinish(): OnFinishFn | undefined {
    return this.onFinishValue;
  }

  // pace per frame in ms
  set speed(newValue: number) {
    this.speedValue = newValue;
  }
  get speed(): number {
    return this.speedValue;
  }

  get isRunning(): boolean {
    return this.isRunningValue;
  }

  get isFinished(): boolean {
    return this.isFinishedValue;
  }

  public play(options: IKeyframeAnimationOptions) {
    const { start, end } = options;

    if (start !== undefined) {
      this.currentFrameValue = start;
    }

    this.targetFrame = end;

    this.isRunningValue = true;
    this.isFinishedValue = false;

    this.unscheduleUpdate();

    this.render();
    this.scheduleUpdate();
  }

  public stop() {
    this.isRunningValue = false;
    this.unscheduleUpdate();
  }

  private unscheduleUpdate() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = undefined;
    }
  }

  private scheduleUpdate() {
    this.timeout = global.setTimeout(() => this.onUpdateFrame(), this.speedValue);
  }

  private render() {
    if (this.onFrameValue) {
      this.onFrameValue(this.currentFrameValue);
    }
  }

  private didFinish() {
    this.isFinishedValue = true;
    this.isRunningValue = false;

    if (this.onFinishValue) {
      this.onFinishValue();
    }
  }

  private onUpdateFrame() {
    this.advanceFrame();

    if (!this.isFinishedValue) {
      this.render();

      // check once again since onFrame() may stop animation
      if (this.isRunningValue) {
        this.scheduleUpdate();
      }
    }
  }

  private advanceFrame() {
    if (this.isFinishedValue) {
      return;
    }

    if (this.currentFrameValue === this.targetFrame) {
      this.didFinish();
    } else if (this.currentFrameValue < this.targetFrame) {
      this.currentFrameValue += 1;
    } else {
      this.currentFrameValue -= 1;
    }
  }
}
