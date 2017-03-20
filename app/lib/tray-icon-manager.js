import assert from 'assert';
import TrayAnimator from './tray-animator';
import TrayIconProvider from './tray-icon-provider';
import { TrayIconType } from '../enums';

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
   * @param {TrayIconProvider} iconProvider 
   * 
   * @memberOf TrayIconManager
   */
  constructor(tray, iconProvider) {
    assert(tray);
    assert(iconProvider);

    this._tray = tray;
    this._iconProvider = iconProvider;
    this._animator = null;
    this._iconType = null;
  }

  /**
   * Destroy manager
   * @memberOf TrayIconManager
   */
  destroy() {
    if(this._animator) {
      this._animator.stop();
      this._animator = null;
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

    // skip animation if:
    // 1. there was no icon set before (which is usually when app starts)
    // 2. unsecured -> securing
    // 3. securing -> unsecured
    const skip = this._iconType === null || 
                 type === TrayIconType.securing || // unsecured -> securing
                 (type === TrayIconType.unsecured && this._iconType === TrayIconType.securing); // securing -> unsecured

    // do not animate if setting icon for the first time
    this._updateType(type, skip);
  }

  /**
   * Get animation for iconType
   * 
   * @param {TrayIconType} type 
   * @returns TrayIconAnimator
   * 
   * @memberOf TrayIconManager
   */
  _animationForType(type) {
    switch(type) {
    case TrayIconType.secured: return this._iconProvider.lockAnimation();
    case TrayIconType.unsecured: return this._iconProvider.unlockAnimation();
    case TrayIconType.securing: return this._iconProvider.unlockAnimation();
    }
  }

  /**
   * Update icon animator with new type
   * 
   * @param {TrayIconType} type
   * @param {boolean} [skipAnimation=false] whether animation should be skipped
   * 
   * @memberOf TrayIconManager
   */
  _updateType(type, skipAnimation = false) {
    assert(TrayIconType.isValid(type));

    let animator = new TrayAnimator(this._tray, this._animationForType(type));

    // destroy existing animator
    if(this._animator) {
      this._animator.stop();
      this._animator = null;
    }

    if(skipAnimation) {
      animator.advanceToEnd();
    } else {
      animator.start();
    }

    this._animator = animator;
    this._iconType = type;
  }

}