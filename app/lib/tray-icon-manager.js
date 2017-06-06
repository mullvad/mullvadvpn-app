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
    assert(tray, 'Tray icon cannot be null');

    const basePath = path.join(path.resolve(__dirname, '..'), 'assets/images/menubar icons');
    let filePath = path.join(basePath, 'lock-{}.png');
    let animation = KeyframeAnimation.fromFilePattern(filePath, [1, 9]);
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
    assert(TrayIconType.isValid(type), 'Invalid icon type');

    // no-op if the same type
    if(this._iconType === type) {
      return;
    }

    let options = { beginFromCurrentState: true };
    if(this._iconType === null) {
      options.advanceTo = 'end';
    }

    switch(type) {
    case TrayIconType.secured:
      this._animation.reverse = false;
      break;
    case TrayIconType.securing:
    case TrayIconType.unsecured:
      this._animation.reverse = true;
      break;
    }

    this._animation.play(options);
    this._iconType = type;
  }

}
