import path from 'path';
import KeyframeAnimation from './keyframe-animation';
import { nativeImage } from 'electron';
import { NativeImage, Tray } from 'electron';

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

export default class TrayIconController {
  _animation?: KeyframeAnimation;
  _iconType: TrayIconType;
  _iconImages: Array<NativeImage> = [];
  _monochromaticIconImages: Array<NativeImage> = [];
  _useMonochromaticIcon: boolean;

  constructor(tray: Tray, initialType: TrayIconType, useMonochromaticIcon: boolean) {
    this._loadImages();
    this._iconType = initialType;
    this._useMonochromaticIcon = useMonochromaticIcon;

    const initialFrame = this._targetFrame();
    const animation = new KeyframeAnimation();
    animation.speed = 100;
    animation.onFrame = (frameNumber) => tray.setImage(this._imageForFrame(frameNumber));
    animation.play({ start: initialFrame, end: initialFrame });

    this._animation = animation;
  }

  dispose() {
    if (this._animation) {
      this._animation.stop();
      this._animation = undefined;
    }
  }

  get iconType(): TrayIconType {
    return this._iconType;
  }

  set useMonochromaticIcon(useMonochromaticIcon: boolean) {
    this._useMonochromaticIcon = useMonochromaticIcon;

    if (this._animation && !this._animation.isRunning) {
      this._animation.play({ end: this._targetFrame() });
    }
  }

  animateToIcon(type: TrayIconType) {
    if (this._iconType === type || !this._animation) {
      return;
    }

    this._iconType = type;

    const animation = this._animation;
    const frame = this._targetFrame();

    animation.play({ end: frame });
  }

  _loadImages() {
    const basePath = path.resolve(path.join(__dirname, '../../assets/images/menubar icons'));
    const frames = Array.from({ length: 10 }, (_, i) => i + 1);

    this._iconImages = frames.map((frame) =>
      nativeImage.createFromPath(path.join(basePath, `lock-${frame}.png`)),
    );

    this._monochromaticIconImages = frames.map((frame) =>
      nativeImage.createFromPath(path.join(basePath, `lock-${frame}Template.png`)),
    );
  }

  _targetFrame(): number {
    switch (this._iconType) {
      case 'unsecured':
        return 0;
      case 'securing':
        return 9;
      case 'secured':
        return 8;
    }
  }

  _imageForFrame(frame: number): NativeImage {
    return this._useMonochromaticIcon
      ? this._monochromaticIconImages[frame]
      : this._iconImages[frame];
  }
}
