// @flow

import path from 'path';
import KeyframeAnimation from './keyframe-animation';
import type { Tray } from 'electron';

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

export default class TrayIconController {
  _animation: ?KeyframeAnimation;
  _iconType: TrayIconType;

  constructor(tray: Tray, initialType: TrayIconType) {
    const animation = this._createAnimation();
    animation.onFrame = (img) => tray.setImage(img);
    animation.reverse = this._isReverseAnimation(initialType);
    animation.play({ advanceTo: 'end' });

    this._animation = animation;
    this._iconType = initialType;
  }

  dispose() {
    if (this._animation) {
      this._animation.stop();
      this._animation = null;
    }
  }

  get iconType(): TrayIconType {
    return this._iconType;
  }

  animateToIcon(type: TrayIconType) {
    if (this._iconType === type || !this._animation) {
      return;
    }

    const animation = this._animation;
    if (type === 'secured') {
      animation.reverse = true;
      animation.play({ beginFromCurrentState: true, startFrame: 8, endFrame: 9 });
    } else {
      animation.reverse = this._isReverseAnimation(type);
      animation.play({ beginFromCurrentState: true });
    }

    this._iconType = type;
  }

  _createAnimation(): KeyframeAnimation {
    const basePath = path.resolve(path.join(__dirname, '../assets/images/menubar icons'));
    const filePath = path.join(basePath, 'lock-{}.png');
    const animation = KeyframeAnimation.fromFilePattern(filePath, [1, 10]);
    animation.speed = 100;
    return animation;
  }

  _isReverseAnimation(type: TrayIconType): boolean {
    return type === 'unsecured';
  }
}
