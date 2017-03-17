import path from 'path';
import { EventEmitter } from 'events';
import TrayAnimation from './tray-animation';
import Enum from './enum';

const menubarIcons = {
  base: path.join(path.resolve(__dirname, '..'), 'assets/images/menubar icons'),
  lock: 'lock-{s}.png'
};

/**
 * Tray icon provider
 * 
 * @export
 * @class TrayIconProvider
 */
export default class TrayIconProvider {

  /**
   * Get lock animation
   * 
   * @param {boolean} [isReverse=false] whether animation should be reversed
   * @returns TrayIconAnimator
   * 
   * @memberOf TrayIconProvider
   */
  lockAnimation(isReverse = false) {
    let filePath = path.join(menubarIcons.base, menubarIcons.lock);
    let animation = TrayAnimation.fromFileSequence(filePath, [1, 9]);
    animation.speed = 100;
    animation.reverse = isReverse;

    return animation;
  }

  /**
   * Get unlock animation
   * 
   * @returns TrayIconAnimator
   * 
   * @memberOf TrayIconProvider
   */
  unlockAnimation() {
    return this.lockAnimation(true);
  }

}