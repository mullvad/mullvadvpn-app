import assert from 'assert';
import { nativeImage } from 'electron';

/**
 * Tray animation descriptor
 * 
 * @export
 * @class TrayAnimation
 */
export default class TrayAnimation {

  /**
   * Set callback called on each frame update
   * 
   * @type {function}
   * @memberOf TrayAnimation
   */
  set onFrame(v) { this._onFrame = v; }

  /**
   * Get callback called on each frame update
   * 
   * @readonly
   * @type {function}
   * @memberOf TrayAnimation
   */
  get onFrame() { this._onFrame; }
  
  /**
   * Set callback called when animation finished
   * 
   * @type {function}
   * @memberOf TrayAnimation
   */
  set onFinish(v) { this._onFinish = v; }

  /**
   * Get callback called when animation finished
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get onFinish() { this._onFinish; }

  /**
   * Set animation pace per frame in ms
   * 
   * @type {number}
   * @memberOf TrayAnimation
   */
  set speed(v) { this._speed = parseInt(v); }

  /**
   * Get animation pace per frame in ms
   * 
   * @readonly
   * @type {number}
   * @memberOf TrayAnimation
   */
  get speed() { return this._speed; }

  /**
   * Set animation repetition
   * @type {bool}
   * 
   * @memberOf TrayAnimation
   */
  set repeat(v) { this._repeat = !!v; }

  /**
   * Get animation repetition
   * 
   * @readonly
   * @type {bool}
   * @memberOf TrayAnimation
   */
  get repeat() { return this._repeat; }

  /**
   * Set animation reversal 
   * @type {bool}
   * @memberOf TrayAnimation
   */
  set reverse(v) { this._reverse = !!v; }

  /**
   * Get animation reversal
   * 
   * @readonly
   * @type {bool}
   * @memberOf TrayAnimation
   */
  get reverse() { return this._repeat; }

  /**
   * Set animation alternation
   * @type {bool}
   * @memberOf TrayAnimation
   */
  set alternate(v) { this._alternate = !!v; }

  /**
   * Get animation alternation
   * 
   * @readonly
   * @type {bool}
   * @memberOf TrayAnimation
   */
  get alternate() { return this._alternate; }

  /**
   * Source array of images
   * 
   * @readonly
   * @type {array}
   * @memberOf TrayAnimation
   */
  get source() { return this._source.slice(); }

  /**
   * Array of NativeImage instances loaded based on source input
   * 
   * @readonly
   * @type {Electron.NativeImage[]}
   * @memberOf TrayAnimation
   */
  get nativeImages() { return this._nativeImages.slice(); }
  
  /**
   * Flag that tells whether animation finished
   * 
   * @readonly
   * @type {bool}
   * @memberOf TrayAnimation
   */
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
   * @memberOf TrayAnimation
   */
  get currentImage() {
    return this._nativeImages[this._currentFrame];
  }

  /**
   * Prepare initial state for animation before running it.
   * @param {object} [options = {}] - animation options
   * @param {number} [options.startFrame] - start frame
   * @param {number} [options.endFrame] - end frame
   * @param {bool} [options.beginFromCurrentState] - continue animation from current state
   * @memberOf TrayAnimation
   */
  play(options = {}) {
    let {startFrame, endFrame, beginFromCurrentState} = options;

    if(startFrame === undefined && endFrame === undefined) {
      this._frameRange = [ 0, this._numFrames - 1 ];
    } else {
      throw 'not implemented';
    }
    
    if(!beginFromCurrentState || this._isFirstRun) {
      this._currentFrame = this._frameRange[this._reverse ? 1 : 0];
    }

    if(this._isFirstRun) {
      this._isFirstRun = false;
    }

    this._isFinished = false;
    
    this._render();
    
    this._unscheduleUpdate();
    this._scheduleUpdate();
  }

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
   * @memberOf TrayAnimation
   */
  _advanceFrame() {
    // do not advance frame when animation is finished
    if(this._isFinished) { return; }

    // advance frame
    let nextFrame = this._nextFrame(this._currentFrame, this._reverse);

    // let animation pick up from current state
    let didReachEnd = (nextFrame < this._frameRange[0] && this._reverse) || // out of bounds but moving into
                      (nextFrame > this._frameRange[1] && !this._reverse); // out of bounds but moving into

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

        // clamp range
        nextFrame = Math.min(Math.max(this._frameRange[0], nextFrame), this._frameRange[1]);
        
        // skip corner frame when alternating by advancing frame once again
        nextFrame = this._nextFrame(nextFrame, this._reverse);
      } else {
        nextFrame = this._frameRange[this._reverse ? 1 : 0];
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

}