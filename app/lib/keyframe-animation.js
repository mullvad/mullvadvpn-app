// @flow
import assert from 'assert';
import { nativeImage } from 'electron';
import type { NativeImage } from 'electron';

export type OnFrameFn = (image: NativeImage) => void;
export type OnFinishFn = (void) => void;
export type KeyframeAnimationOptions = {
  startFrame?: number,
  endFrame?: number,
  beginFromCurrentState?: boolean,
  advanceTo?: 'end'
};

/**
 * Keyframe animation
 *
 * @export
 * @class KeyframeAnimation
 */
export default class KeyframeAnimation {

  _speed: number = 200; // ms
  _repeat: boolean = false;
  _reverse: boolean = false;
  _alternate: boolean = false;

  _onFrame: ?OnFrameFn;
  _onFinish: ?OnFinishFn;

  _nativeImages: Array<NativeImage>;
  _frameRange: Array<number>;
  _numFrames: number;
  _currentFrame: number = 0;

  _isRunning: boolean = false;
  _isFinished: boolean = false;
  _isFirstRun: boolean = true;

  _timeout = null;

  /**
   * Set callback called on each frame update
   *
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  set onFrame(newValue: ?OnFrameFn) { this._onFrame = newValue; }

  /**
   * Get callback called on each frame update
   *
   * @readonly
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  get onFrame(): ?OnFrameFn { this._onFrame; }

  /**
   * Set callback called when animation finished
   *
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  set onFinish(newValue: ?OnFinishFn) { this._onFinish = newValue; }

  /**
   * Get callback called when animation finished
   *
   * @readonly
   *
   * @memberOf KeyframeAnimation
   */
  get onFinish(): ?OnFinishFn { this._onFinish; }

  /**
   * Set animation pace per frame in ms
   *
   * @type {number}
   * @memberOf KeyframeAnimation
   */
  set speed(newValue: number) { this._speed = parseInt(newValue); }

  /**
   * Get animation pace per frame in ms
   *
   * @readonly
   * @type {number}
   * @memberOf KeyframeAnimation
   */
  get speed(): number { return this._speed; }

  /**
   * Set animation repetition
   * @type {bool}
   *
   * @memberOf KeyframeAnimation
   */
  set repeat(newValue: boolean) { this._repeat = !!newValue; }

  /**
   * Get animation repetition
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get repeat(): boolean { return this._repeat; }

  /**
   * Set animation reversal
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  set reverse(newValue: boolean) { this._reverse = !!newValue; }

  /**
   * Get animation reversal
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get reverse(): boolean { return this._repeat; }

  /**
   * Set animation alternation
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  set alternate(newValue: boolean) { this._alternate = !!newValue; }

  /**
   * Get animation alternation
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get alternate(): boolean { return this._alternate; }

  /**
   * Array of NativeImage instances loaded based on source input
   *
   * @readonly
   * @type {Array<NativeImage>}
   * @memberOf KeyframeAnimation
   */
  get nativeImages(): Array<NativeImage> { return this._nativeImages.slice(); }

  /**
   * Flag that tells whether animation finished
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get isFinished(): boolean { return this._isFinished; }

  /**
   * Create animation from files matching filename pattern
   *
   * @static
   * @param {string}        filePattern - file name pattern where {} is replaced with index
   * @param {Array<number>} range       - sequence range [start, end]
   *
   * @memberOf KeyframeAnimation
   * @return {KeyframeAnimation}
   */
  static fromFilePattern(filePattern: string, range: Array<number>): KeyframeAnimation {
    assert(range.length === 2 && range[0] < range[1], 'the animation range is invalid');
    const images: Array<NativeImage> = [];
    for(let i = range[0]; i <= range[1]; i++) {
      const filePath = filePattern.replace('{}', i.toString());
      const image = nativeImage.createFromPath(filePath);
      images.push(image);
    }
    return new KeyframeAnimation(images);
  }

  /**
   * Create animation from file sequence
   *
   * @static
   * @param {Array<string>} files - file paths
   * @returns {KeyframeAnimation}
   *
   * @memberof KeyframeAnimation
   */
  static fromFileSequence(files: Array<string>): KeyframeAnimation {
    const images: Array<NativeImage> = files.map(filePath => nativeImage.createFromPath(filePath));
    return new KeyframeAnimation(images);
  }

  /**
   * Create an instance of KeyframeAnimation
   * @param {Array<NativeImage>} images - an array of instances of NativeImage
   *
   * @memberOf KeyframeAnimation
   */
  constructor(images: Array<NativeImage>) {
    const len = images.length;

    assert(len > 0, 'too few images in animation');

    this._nativeImages = images.slice();
    this._numFrames = len;
    this._frameRange = [0, len];
  }

  /**
   * Get current sprite
   *
   * @readonly
   * @type {NativeImage}
   * @memberOf KeyframeAnimation
   */
  get currentImage(): NativeImage {
    return this._nativeImages[this._currentFrame];
  }

  /**
   * Start animation
   *
   * @param {object} [options = {}]                  - animation options
   * @param {number} [options.startFrame]            - start frame
   * @param {number} [options.endFrame]              - end frame
   * @param {bool}   [options.beginFromCurrentState] - continue animation from current state
   * @param {string} [options.advanceTo]             - resets current frame. (possible values: end)
   * @memberOf KeyframeAnimation
   */
  play(options: KeyframeAnimationOptions = {}) {
    let { startFrame, endFrame, beginFromCurrentState, advanceTo } = options;

    if(startFrame !== undefined && endFrame !== undefined) {
      assert(startFrame >= 0 && startFrame < this._numFrames, 'start frame is invalid');
      assert(endFrame >= 0 && endFrame < this._numFrames, 'end frame is invalid');

      if(startFrame < endFrame) {
        this._frameRange = [ startFrame, endFrame ];
      } else {
        this._frameRange = [ endFrame, startFrame ];
      }
    } else {
      this._frameRange = [ 0, this._numFrames - 1 ];
    }

    if(!beginFromCurrentState || this._isFirstRun) {
      this._currentFrame = this._frameRange[this._reverse ? 1 : 0];
    }

    if(this._isFirstRun) {
      this._isFirstRun = false;
    }

    if(advanceTo === 'end') {
      this._currentFrame = this._frameRange[this._reverse ? 0 : 1];
    }

    this._isRunning = true;
    this._isFinished = false;

    this._unscheduleUpdate();

    this._render();
    this._scheduleUpdate();
  }

  /**
   * Stop animation
   * @memberOf KeyframeAnimation
   */
  stop() {
    this._isRunning = false;
    this._unscheduleUpdate();
  }

  /**
   * Cancel timer for next animation frame
   *
   * @private
   * @memberof KeyframeAnimation
   */
  _unscheduleUpdate() {
    if(this._timeout) {
      clearTimeout(this._timeout);
      this._timeout = null;
    }
  }

  /**
   * Schedule timer for next animation frame
   *
   * @private
   * @memberof KeyframeAnimation
   */
  _scheduleUpdate() {
    this._timeout = setTimeout(() => this._onUpdateFrame(), this._speed);
  }

  /**
   * Call delegate to render frame
   *
   * @private
   * @memberof KeyframeAnimation
   */
  _render() {
    if(this._onFrame) {
      this._onFrame(this._nativeImages[this._currentFrame]);
    }
  }

  /**
   * Mark animation finished and notify delegate.
   *
   * @private
   * @memberof KeyframeAnimation
   */
  _didFinish() {
    this._isFinished = true;

    if(this._onFinish) {
      this._onFinish();
    }
  }

  /**
   * Animation frame lifecycle.
   *
   * @private
   * @memberof KeyframeAnimation
   */
  _onUpdateFrame() {
    this._advanceFrame();

    if(this._isFinished) {
      // mark animation as not running when finished
      this._isRunning = false;
    } else {
      this._render();

      // check once again since onFrame() may stop animation
      if(this._isRunning) {
        this._scheduleUpdate();
      }
    }
  }

  /**
   * Advance animation frame
   * @memberOf KeyframeAnimation
   */
  _advanceFrame() {
    // do not advance frame when animation is finished
    if(this._isFinished) { return; }

    // advance frame
    let didReachEnd = this._currentFrame === this._frameRange[this._reverse ? 0 : 1];

    // did reach end?
    if(didReachEnd) {
      // mark animation as finished if it's not repeating
      if(!this._repeat) {
        this._didFinish();
        return;
      }

      // change animation direction if marked for alternation
      if(this._alternate) {
        this._reverse = !this._reverse;

        this._currentFrame = this._nextFrame(this._currentFrame, this._frameRange, this._reverse);
      } else {
        this._currentFrame = this._frameRange[this._reverse ? 1 : 0];
      }
    } else {
      this._currentFrame = this._nextFrame(this._currentFrame, this._frameRange, this._reverse);
    }
  }

  /**
   * Calculate next frame
   * @private
   * @param {number}        cur        - current frame
   * @param {Array<number>} frameRange - frame range
   * @param {bool}          isReverse  - reverse sequence direction?
   * @returns {number}
   *
   * @memberOf KeyframeAnimation
   */
  _nextFrame(cur: number, frameRange: Array<number>, isReverse: boolean): number {
    if(isReverse) {
      if(cur < frameRange[0]) {
        return cur + 1;
      } else if(cur > frameRange[0]) {
        return cur - 1;
      }
    } else {
      if(cur > frameRange[1]) {
        return cur - 1;
      } else if(cur < frameRange[1]) {
        return cur + 1;
      }
    }
    return cur;
  }

}
