import { nativeImage, NativeImage, Tray } from 'electron';
import path from 'path';
import KeyframeAnimation from './keyframe-animation';

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

export default class TrayIconController {
  private animation?: KeyframeAnimation;
  private iconImages: NativeImage[] = [];

  constructor(
    tray: Tray,
    private iconTypeValue: TrayIconType,
    private useMonochromaticIconValue: boolean,
  ) {
    this.loadImages();

    const initialFrame = this.targetFrame();
    const animation = new KeyframeAnimation();
    animation.speed = 100;
    animation.onFrame = (frameNumber) => tray.setImage(this.iconImages[frameNumber]);
    animation.play({ start: initialFrame, end: initialFrame });

    this.animation = animation;
  }

  public dispose() {
    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }
  }

  get iconType(): TrayIconType {
    return this.iconTypeValue;
  }

  set useMonochromaticIcon(useMonochromaticIcon: boolean) {
    this.useMonochromaticIconValue = useMonochromaticIcon;
    this.loadImages();

    if (this.animation && !this.animation.isRunning) {
      this.animation.play({ end: this.targetFrame() });
    }
  }

  public animateToIcon(type: TrayIconType) {
    if (this.iconTypeValue === type || !this.animation) {
      return;
    }

    this.iconTypeValue = type;

    const animation = this.animation;
    const frame = this.targetFrame();

    animation.play({ end: frame });
  }

  private loadImages() {
    const basePath = path.resolve(path.join(__dirname, '../../assets/images/menubar icons'));
    const frames = Array.from({ length: 10 }, (_, i) => i + 1);

    const suffix = this.imageSuffix();
    const extension = process.platform === 'win32' ? 'ico' : 'png';

    this.iconImages = frames.map((frame) =>
      nativeImage.createFromPath(path.join(basePath, `lock-${frame}${suffix}.${extension}`)),
    );
  }

  private imageSuffix() {
    if (this.useMonochromaticIconValue && process.platform === 'darwin') {
      return 'Template';
    } else {
      return '';
    }
  }

  private targetFrame(): number {
    switch (this.iconTypeValue) {
      case 'unsecured':
        return 0;
      case 'securing':
        return 9;
      case 'secured':
        return 8;
    }
  }
}
