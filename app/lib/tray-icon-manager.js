import assert from 'assert';
import path from 'path';
import { TrayIconType } from '../enums';
import KeyframeAnimation from './keyframe-animation';

/**
 * Tray icon manager
 * 
 * @export
 * @class TrayIconManager
 */
export default class TrayIconManager {
  
  /**
   * Creates an instance of TrayIconManager.
   * @param {Electron.Tray} tray 
   * 
   * @memberOf TrayIconManager
   */
  constructor(tray) {
    assert(tray);

    const basePath = path.join(path.resolve(__dirname, '..'), 'assets/images/menubar icons');
    let filePath = path.join(basePath, 'lock-{s}.png');
    let animation = KeyframeAnimation.fromFileSequence(filePath, [1, 9]);
    animation.onFrame = (img) => tray.setImage(img);
    animation.speed = 100;

    this._animation = animation;
    this._iconType = null;
  }

  /**
   * Destroy manager
   * @memberOf TrayIconManager
   */
  destroy() {
    if(this._animation) {
      this._animation.stop();
      this._animation = null;
    }
    this._iconType = null;
  }

  /**
   * Get current icon type
   * @type {TrayIconType}
   * @memberOf TrayIconManager
   */
  get iconType() { 
    return this._iconType; 
  }

  /**
   * Set current icon type
   * @type {TrayIconType}
   * @memberOf TrayIconManager
   */
  set iconType(type) {
    this._updateIconType(type);
  }

  /**
   * Set current icon type with options
   * 
   * @param {TrayIconType} type          - new icon type
   * @param {bool}         skipAnimation - pass true to skip animation to last frame. Has no effect on repeating animations.
   * @returns 
   * 
   * @memberOf TrayIconManager
   */
  _updateIconType(type) {
    // no-op if same animator requested
    if(this._iconType === type) { return; }

    this._updateType(type);
  }

  /**
   * Update icon animator with new type
   * 
   * @param {TrayIconType} type
   * 
   * @memberOf TrayIconManager
   */
  _updateType(type) {
    assert(TrayIconType.isValid(type));

    let options = { beginFromCurrentState: true };

    switch(type) {
    case TrayIconType.secured:
      this._animation.reverse = false;
      break;
    case TrayIconType.securing:
    case TrayIconType.unsecured:
      this._animation.reverse = true;
      break;
    }

    if(this._iconType === null) {
      options.advanceTo = 'end';
    }
    
    this._animation.play(options);

    // if(skipAnimation) {
    //   animator.advanceToEnd();
    // } else {
    //   animator.start();
    // }

    this._iconType = type;
  }

}