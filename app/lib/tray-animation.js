import assert from 'assert';
import { nativeImage } from 'electron';

/**
 * Tray animation descriptor
 * 
 * @export
 * @class TrayAnimation
 */
export class TrayAnimation {

  /**
   * Set animation pace per frame in ms
   * 
   * @memberOf TrayAnimation
   */
  set speed(v) { this._speed = parseInt(v); }

  /**
   * Get animation pace per frame in ms
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get speed() { return this._speed; }

  /**
   * Set animation repetition
   * @memberOf TrayAnimation
   */
  set repeat(v) { this._repeat = !!v; }

  /**
   * Get animation repetition
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get repeat() { return this._repeat; }

  /**
   * Set animation reversal 
   * @memberOf TrayAnimation
   */
  set reverse(v) { this._reverse = !!v; }

  /**
   * Get animation reversal
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get reverse() { return this._repeat; }

  /**
   * Set animation alternation
   * 
   * @memberOf TrayAnimation
   */
  set alternate(v) { this._alternate = !!v; }

  /**
   * Get animation alternation
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get alternate() { return this._alternate; }

  /**
   * Source array of images
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get source() { return this._source.slice(); }

  /**
   * Array of NativeImage instances loaded based on source input
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
  get nativeImages() { return this._nativeImages.slice(); }
  
  /**
   * Flag that tells whether animation finished
   * 
   * @readonly
   * 
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
    this._isFinished = false;
  }

  /**
   * Get current sprite
   * 
   * @readonly
   * 
   * @memberOf TrayAnimation
   */
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

  /**
   * Advance animation to the start
   * This method respects animation reversal
   * 
   * @memberOf TrayAnimation
   */
  advanceToStart() {
    this._currentFrame = this._firstFrame(this._reverse);
  }

  /**
   * Advance animation to the end
   * This method respects animation reversal
   * 
   * @memberOf TrayAnimation
   */
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