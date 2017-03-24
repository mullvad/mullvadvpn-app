import assert from 'assert';
import { nativeImage } from 'electron';

/**
 * Keyframe animation
 *
 * @export
 * @class KeyframeAnimation
 */
export default class KeyframeAnimation {

  /**
   * Set callback called on each frame update
   *
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  set onFrame(v) { this._onFrame = v; }

  /**
   * Get callback called on each frame update
   *
   * @readonly
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  get onFrame() { this._onFrame; }

  /**
   * Set callback called when animation finished
   *
   * @type {function}
   * @memberOf KeyframeAnimation
   */
  set onFinish(v) { this._onFinish = v; }

  /**
   * Get callback called when animation finished
   *
   * @readonly
   *
   * @memberOf KeyframeAnimation
   */
  get onFinish() { this._onFinish; }

  /**
   * Set animation pace per frame in ms
   *
   * @type {number}
   * @memberOf KeyframeAnimation
   */
  set speed(v) { this._speed = parseInt(v); }

  /**
   * Get animation pace per frame in ms
   *
   * @readonly
   * @type {number}
   * @memberOf KeyframeAnimation
   */
  get speed() { return this._speed; }

  /**
   * Set animation repetition
   * @type {bool}
   *
   * @memberOf KeyframeAnimation
   */
  set repeat(v) { this._repeat = !!v; }

  /**
   * Get animation repetition
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get repeat() { return this._repeat; }

  /**
   * Set animation reversal
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  set reverse(v) { this._reverse = !!v; }

  /**
   * Get animation reversal
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get reverse() { return this._repeat; }

  /**
   * Set animation alternation
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  set alternate(v) { this._alternate = !!v; }

  /**
   * Get animation alternation
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get alternate() { return this._alternate; }

  /**
   * Source array of images
   *
   * @readonly
   * @type {array}
   * @memberOf KeyframeAnimation
   */
  get source() { return this._source.slice(); }

  /**
   * Array of NativeImage instances loaded based on source input
   *
   * @readonly
   * @type {Electron.NativeImage[]}
   * @memberOf KeyframeAnimation
   */
  get nativeImages() { return this._nativeImages.slice(); }

  /**
   * Flag that tells whether animation finished
   *
   * @readonly
   * @type {bool}
   * @memberOf KeyframeAnimation
   */
  get isFinished() { return this._isFinished; }

  /**
   * Create animation using file sequence
   *
   * @static
   * @param {string}   filePattern - file name pattern where {s} is replaced with index
   * @param {number[]} range       - sequence range [start, end]
   *
   * @memberOf KeyframeAnimation
   * @return {KeyframeAnimation}
   */
  static fromFileSequence(filePattern, range) {
    assert(range.length === 2 && range[0] < range[1]);

    let images = [];
    for(let i = range[0]; i <= range[1]; i++) {
      images.push(filePattern.replace('{s}', i));
    }

    return new KeyframeAnimation(images);
  }

  /**
   * Creates an instance of KeyframeAnimation.
   * @param {string[]} images
   *
   * @memberOf KeyframeAnimation
   */
  constructor(images) {
    assert(images.length > 0);

    this._source = images.slice();
    this._nativeImages = images.map((pathOrNativeImage) => {
      if(typeof(pathOrNativeImage) === 'string') {
        return nativeImage.createFromPath(pathOrNativeImage);
      } else if((pathOrNativeImage + '') === '[object NativeImage]') {
        return pathOrNativeImage;
      }
      return nativeImage.createEmpty();
    });

    this._speed = 200; // ms
    this._repeat = false;
    this._reverse = false;
    this._alternate = false;

    this._numFrames = images.length;
    this._currentFrame = 0;
    this._frameRange = [0, this._numFrames];
    this._isFinished = false;

    this._isFirstRun = true;
  }

  /**
   * Get current sprite
   *
   * @readonly
   * @type {Electron.NativeImage}
   * @memberOf KeyframeAnimation
   */
  get currentImage() {
    return this._nativeImages[this._currentFrame];
  }

  /**
   * Prepare initial state for animation before running it.
   * @param {object} [options = {}]                - animation options
   * @param {number} [options.startFrame]          - start frame
   * @param {number} [options.endFrame]            - end frame
   * @param {bool} [options.beginFromCurrentState] - continue animation from current state
   * @param {string} [options.advanceTo]           - resets current frame. (possible values: end)
   * @memberOf KeyframeAnimation
   */
  play(options = {}) {
    let { startFrame, endFrame, beginFromCurrentState, advanceTo } = options;

    if(startFrame !== undefined && endFrame !== undefined) {
      assert(startFrame >= 0 && startFrame < this._numFrames);
      assert(endFrame >= 0 && endFrame < this._numFrames);

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
    this._unscheduleUpdate();
  }

  _unscheduleUpdate() {
    if(this._timeout) {
      clearTimeout(this._timeout);
      this._timeout = null;
    }
  }

  _scheduleUpdate() {
    this._timeout = setTimeout(::this._onUpdateFrame, this._speed);
  }

  _render() {
    if(this._onFrame) {
      this._onFrame(this._nativeImages[this._currentFrame]);
    }
  }

  _didFinish() {
    if(this._onFinish) {
      this._onFinish();
    }
  }

  _onUpdateFrame() {
    this._advanceFrame();

    if(!this._isFinished) {
      this._render();
      this._scheduleUpdate();
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
      // mark animation as finished if it's not marked as repeating
      if(!this._repeat) {
        this._isFinished = true;

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
   * @param {number}   cur        - current frame
   * @param {number[]} frameRange - frame range
   * @param {bool}     isReverse  - reverse sequence direction?
   * @returns {number}
   *
   * @memberOf KeyframeAnimation
   */
  _nextFrame(cur, frameRange, isReverse) {
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