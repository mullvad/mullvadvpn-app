import assert from 'assert';
import { TrayAnimator } from './tray-animator';
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
    let animator;
    assert(TrayIconType.isValid(type));

    // no-op if same animator requested
    if(this._iconType === type) { return; }
    
    // destroy existing animator
    if(this._animator) {
      this._animator.stop();
      this._animator = null;
    }

    // do not animate if setting icon for the first time
    const skipAnimation = this._iconType === null;

    switch(type) {
    case TrayIconType.secured:
      animator = new TrayAnimator(this._tray, this._iconProvider.lockAnimation());
      if(skipAnimation) {
        animator.advanceToEnd();
      } else {
        animator.start();
      }
      break;

    case TrayIconType.unsecured:
      animator = new TrayAnimator(this._tray, this._iconProvider.unlockAnimation());
      if(skipAnimation) {
        animator.advanceToStart();
      } else {
        animator.start();
      }
      break;

    case TrayIconType.securing:
      animator = new TrayAnimator(this._tray, this._iconProvider.spinnerAnimation());
      animator.start();
      break;
    }

    this._animator = animator;
    this._iconType = type;
  }

}