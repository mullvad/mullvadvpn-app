import assert from 'assert';

/**
 * Tray icon animator 
 * @class TrayAnimator
 */
export default class TrayAnimator {
  
  /**
   * Whether animator has started.
   * @readonly
   * @memberOf TrayAnimator
   */
  get isStarted() { return this._started; }

  /**
   * Creates an instance of TrayAnimator.
   * @param {Electron.Tray} tray      - an instance of Tray
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
  
  advanceToStart() {
    this._animation.advanceToStart();
    this._updateTrayIcon();
  }

  advanceToEnd() {
    this._animation.advanceToEnd();
    this._updateTrayIcon();
  }

  /**
   * Start animating
   * @memberOf TrayAnimator
   */
  start() {
    if(this._started) { return; }

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
    return setTimeout(::this._updateAnimationFrame, this._animation.speed);
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
