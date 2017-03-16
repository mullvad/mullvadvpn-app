import assert from 'assert';
import fs from 'fs';
import path from 'path';
import { nativeImage } from 'electron';

/**
 * Tray animation descriptor
 * 
 * @export
 * @class TrayAnimation
 * @property {number}               speed        - speed per frame
 * @property {bool}                 repeat       - whether to repeat animation
 * @property {bool}                 reverse      - play in reverse
 * @property {bool}                 alternate    - whether to alternate sequence when reached the end of animation
 * @property {string[]}             source       - image source
 * @property {electron.NativeImage} nativeImages - a sequence of native images
 * @property {bool}                 isFinished   - whether animation sequence is finished (repeating animation never finish)
 */
export class TrayAnimation {

  set speed(v) { this._speed = parseInt(v); }
  get speed() { return this._speed; }

  set repeat(v) { this._repeat = !!v; }
  get repeat() { return this._repeat; }

  set reverse(v) { this._reverse = !!v; }
  get reverse() { return this._repeat; }

  set alternate(v) { this._alternate = !!v; }
  get alternate() { return this._alternate; }

  get source() { return this._source.slice(); }
  get nativeImages() { return this._nativeImages.slice(); }

  get isFinished() { return this._isFinished; }
  
  /**
   * Create animation using file sequence
   * 
   * @static
   * @param {string}   filePattern - file name pattern where {s} is replaced with index 
   * @param {number[]} range       - sequence range [start, end]
   * 
   * @memberOf TrayAnimation
   * @return {TrayAnimation}
   */
  static fromFileSequence(filePattern, range) {
    assert(range.length === 2 && range[0] < range[1]);

    let images = [];
    for(let i = range[0]; i <= range[1]; i++) {
      images.push(filePattern.replace('{s}', i));
    }

    return new TrayAnimation(images);
  }

  /**
   * Creates an instance of TrayAnimation.
   * @param {string[]} images 
   * 
   * @memberOf TrayAnimation
   */
  constructor(images) {
    assert(images.length > 0);
    
    this._source = images.slice();
    this._nativeImages = images.map((pathOrNativeImage) => {
      if(typeof(pathOrNativeImage) === 'string') {
        return nativeImage.createFromPath(pathOrNativeImage);
      } else if(typeof(pathOrNativeImage) === 'NativeImage') {
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
    this._isFinished = false;
  }

  get currentImage() {
    return this._nativeImages[this._currentFrame];
  }

  /**
   * Prepare initial state for animation before running it.
   * @memberOf TrayAnimation
   */
  prepare() {
    this._currentFrame = this._firstFrame(this._reverse);
  }

  advanceToStart() {
    this._currentFrame = this._firstFrame(this._reverse);
  }

  advanceToEnd() {
    this._currentFrame = this._lastFrame(this._reverse);
  }

  /**
   * Advance animation frame
   * @memberOf TrayAnimation
   */
  advanceFrame() {
    // do not advance frame when animation is finished
    if(this._isFinished) { return; }

    // advance frame
    let nextFrame = this._nextFrame(this._currentFrame, this._reverse);

    // did reach end?
    if(nextFrame < 0 || nextFrame >= this._numFrames) {
      // mark animation as finished if it's not marked as repeating
      if(!this._repeat) {
        this._isFinished = true;
        return;
      }

      // change animation direction if marked for alternation
      if(this._alternate) {
        this._reverse = !this._reverse;

        // clamp range
        nextFrame = Math.min(Math.max(0, nextFrame), this._numFrames - 1);
        
        // skip corner frame when alternating by advancing frame once again
        nextFrame = this._nextFrame(nextFrame, this._reverse);
      } else {
        nextFrame = this._firstFrame(this._reverse);
      }
    }
    this._currentFrame = nextFrame;
  }

  /**
   * Calculate next frame
   * @private
   * @param {number} cur       - current frame
   * @param {bool}   isReverse - reverse sequence direction?
   * @returns {number}
   * 
   * @memberOf TrayAnimation
   */
  _nextFrame(cur, isReverse) {
    return cur + (isReverse ? -1 : 1);
  }

  /**
   * Get first frame of animation
   * 
   * @param {bool} isReverse reverse animation?
   * @returns {number}
   * 
   * @memberOf TrayAnimation
   */
  _firstFrame(isReverse) {
    return isReverse ? this._numFrames - 1 : 0;
  }

  /**
   * Get last frame of animation
   * 
   * @param {bool} isReverse reverse animation?
   * @returns {number}
   * 
   * @memberOf TrayAnimation
   */
  _lastFrame(isReverse) {
    return isReverse ? 0 : this._numFrames - 1;
  }

}

/**
 * Tray icon animator 
 * @class TrayAnimator
 */
export class TrayAnimator {
  
  /**
   * Whether animator has started.
   * @readonly
   * @memberOf TrayAnimator
   */
  get isStarted() { return this._started; }

  /**
   * Creates an instance of TrayAnimator.
   * @param {electron.Tray} tray      - an instance of Tray
   * @param {TrayAnimation} animation - an instance of TrayAnimation
   * 
   * @memberOf TrayAnimator
   */
  constructor(tray, animation) {
    assert(tray);
    assert(animation);

    this._tray = tray;
    this._animation = animation;
    this._started = false;
    this._timer = null;
  }

  /**
   * Start animating
   * @memberOf TrayAnimator
   */
  start() {
    if(this._started) { return; };

    this._timer = this._nextFrame();
    this._started = true;

    // prepare animation
    this._animation.prepare();

    // update from initial frame
    this._updateTrayIcon();
  }

  /**
   * Stop animating
   * @memberOf TrayAnimator
   */
  stop() {
    if(!this._started) { return; }

    this._started = false;
    
    clearTimeout(this._timer);
    this._timer = null;
  }

  /**
   * Schedules next animation frame
   * @returns {number} timer ID
   * @memberOf TrayAnimator
   */
  _nextFrame() {
    return setTimeout(::this._updateAnimationFrame, this._animation.speed)
  }

  /**
   * Updates animation frame
   * @memberOf TrayAnimator
   */
  _updateAnimationFrame() {
    if(!this._started) { return; }

    this._animation.advanceFrame();
    this._updateTrayIcon();

    if(!this._animation.isFinished) {
      this._nextFrame();
    }
  }

  /**
   * Update tray icon with current frame
   * @memberOf TrayAnimator
   */
  _updateTrayIcon() {
    this._tray.setImage(this._animation.currentImage);
  }

}
