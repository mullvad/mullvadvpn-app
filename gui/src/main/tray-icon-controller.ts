import { nativeImage, NativeImage, Tray } from 'electron';
import path from 'path';
import KeyframeAnimation from './keyframe-animation';

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

export default class TrayIconController {
  private animation?: KeyframeAnimation;
  private iconImages: NativeImage[] = [];
  private iconTypeValue?: TrayIconType;

  constructor(private tray: Tray, private useMonochromaticIconValue: boolean) {
    this.loadImages();
  }

  public dispose() {
    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }
  }

  get iconType(): TrayIconType | undefined {
    return this.iconTypeValue;
  }

  set useMonochromaticIcon(useMonochromaticIcon: boolean) {
    this.useMonochromaticIconValue = useMonochromaticIcon;
    this.loadImages();

    const targetFrame = this.targetFrame();
    if (this.animation && !this.animation.isRunning && targetFrame !== undefined) {
      this.animation.play({ end: targetFrame });
    }
  }

  public animateToIcon(type: TrayIconType) {
    if (this.iconTypeValue === type) {
      return;
    }

    this.iconTypeValue = type;
    const frame = this.targetFrame();

    if (frame !== undefined) {
      if (!this.animation) {
        this.initAnimation(frame);
      } else {
        this.animation.play({ end: frame });
      }
    }
  }

  private initAnimation(startFrame: number) {
    const animation = new KeyframeAnimation();
    animation.onFrame = (frameNumber) => this.tray.setImage(this.iconImages[frameNumber]);
    animation.speed = 100;
    animation.currentFrame = startFrame;
    this.animation = animation;
  }

  private loadImages() {
    const frames = Array.from({ length: 10 }, (_, i) => i + 1);
    this.iconImages = frames.map((frame) => nativeImage.createFromPath(this.getImagePath(frame)));
  }

  private getImagePath(frame: number) {
    const basePath = path.resolve(path.join(__dirname, '../../assets/images/menubar icons'));
    const extension = process.platform === 'win32' ? 'ico' : 'png';
    let suffix = '';
    if (this.useMonochromaticIconValue) {
      suffix = process.platform === 'darwin' ? 'Template' : '_white';
    }

    return path.join(basePath, process.platform, `lock-${frame}${suffix}.${extension}`);
  }

  private targetFrame(): number | undefined {
    switch (this.iconTypeValue) {
      case 'unsecured':
        return 0;
      case 'securing':
        return 9;
      case 'secured':
        return 8;
      default:
        return undefined;
    }
  }
}
