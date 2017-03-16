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
   * @param {electron.Tray} tray 
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
    
    iconProvider.on(TrayIconProvider.EventType.themeChanged, this._onThemeChange);
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
    this._iconProvider.removeListener(TrayIconProvider.EventType.themeChanged, this._onThemeChange);
  }

  /**
   * Event handler for notification when menubar theme is changed.
   * @memberOf TrayIconManager
   */
  _onThemeChange = () => {
    this._updateType(this._iconType, true);
  }

  /**
   * Get current icon type
   * @memberOf TrayIconManager
   */
  get iconType() { 
    return this._iconType; 
  }

  /**
   * Set current icon type
   * @memberOf TrayIconManager
   */
  set iconType(type) {
    // no-op if same animator requested
    if(this._iconType === type) { return; }

    // do not animate if setting icon for the first time
    const skipAnimation = this._iconType === null;

    this._updateType(type, skipAnimation);
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
    case TrayIconType.securing: return this._iconProvider.spinnerAnimation();
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

    switch(type) {
    case TrayIconType.secured:
      if(skipAnimation) {
        animator.advanceToEnd();
      } else {
        animator.start();
      }
      break;

    case TrayIconType.unsecured:
      if(skipAnimation) {
        animator.advanceToStart();
      } else {
        animator.start();
      }
      break;

    case TrayIconType.securing:
      animator.start();
      break;
    }

    this._animator = animator;
    this._iconType = type;
  }

}