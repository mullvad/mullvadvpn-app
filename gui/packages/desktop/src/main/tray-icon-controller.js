// @flow

import path from 'path';
import KeyframeAnimation from './keyframe-animation';
import { nativeImage } from 'electron';
import type { NativeImage, Tray } from 'electron';

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

export default class TrayIconController {
  _animation: ?KeyframeAnimation;
  _iconType: TrayIconType;
  _iconImages: Array<NativeImage>;

  constructor(tray: Tray, initialType: TrayIconType) {
    this._loadImages();

    const animation = new KeyframeAnimation(this._iconImages.length);
    animation.speed = 100;
    animation.onFrame = (frameNumber) => tray.setImage(this._iconImages[frameNumber]);
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

  _loadImages() {
    const basePath = path.resolve(path.join(__dirname, '../assets/images/menubar icons'));
    const frames = Array.from({ length: 10 }, (_, i) => i + 1);

    this._iconImages = frames.map((frame) =>
      nativeImage.createFromPath(path.join(basePath, `lock-${frame}.png`)),
    );
  }

  _isReverseAnimation(type: TrayIconType): boolean {
    return type === 'unsecured';
  }
}
