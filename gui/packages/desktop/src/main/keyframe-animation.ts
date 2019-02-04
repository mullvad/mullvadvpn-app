export type OnFrameFn = (frame: number) => void;
export type OnFinishFn = () => void;

export interface IKeyframeAnimationOptions {
  start?: number;
  end: number;
}
export type KeyframeAnimationRange = [number, number];

export default class KeyframeAnimation {
  private speedValue: number = 200; // ms

  private onFrameValue?: OnFrameFn;
  private onFinishValue?: OnFinishFn;

  private currentFrame: number = 0;
  private targetFrame: number = 0;

  private isRunningValue: boolean = false;
  private isFinishedValue: boolean = false;

  private timeout?: NodeJS.Timeout;

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
      this.currentFrame = start;
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
    this.timeout = setTimeout(() => this.onUpdateFrame(), this.speedValue);
  }

  private render() {
    if (this.onFrameValue) {
      this.onFrameValue(this.currentFrame);
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

    if (this.currentFrame === this.targetFrame) {
      this.didFinish();
    } else if (this.currentFrame < this.targetFrame) {
      this.currentFrame += 1;
    } else {
      this.currentFrame -= 1;
    }
  }
}
