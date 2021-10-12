import { exec as execAsync } from 'child_process';
import { nativeImage, NativeImage, Tray } from 'electron';
import path from 'path';
import { promisify } from 'util';
import log from '../shared/logging';
import KeyframeAnimation from './keyframe-animation';

const exec = promisify(execAsync);

export type TrayIconType = 'unsecured' | 'securing' | 'secured';

type IconSets = {
  regular: NativeImage[];
  template: NativeImage[];
  white: NativeImage[];
  black: NativeImage[];
};

export default class TrayIconController {
  private animation?: KeyframeAnimation;
  private iconSets: IconSets = { regular: [], template: [], white: [], black: [] };
  private iconSet: NativeImage[] = [];

  constructor(
    private tray: Tray,
    private iconTypeValue: TrayIconType,
    private useMonochromaticIconValue: boolean,
  ) {
    this.loadImages();
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

  public async updateTheme() {
    if (this.useMonochromaticIconValue) {
      switch (process.platform) {
        case 'darwin':
          this.iconSet = this.iconSets.template;
          break;
        case 'win32': {
          if (await this.getSystemUsesLightTheme()) {
            this.iconSet = this.iconSets.black;
          } else {
            this.iconSet = this.iconSets.white;
          }
          break;
        }
        case 'linux':
        default:
          this.iconSet = this.iconSets.white;
          break;
      }
    } else {
      this.iconSet = this.iconSets.regular;
    }

    if (this.animation === undefined) {
      this.initAnimation();
    } else if (!this.animation.isRunning) {
      this.animation.play({ end: this.targetFrame() });
    }
  }

  public async setUseMonochromaticIcon(useMonochromaticIcon: boolean) {
    this.useMonochromaticIconValue = useMonochromaticIcon;
    await this.updateTheme();
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

  private initAnimation() {
    const initialFrame = this.targetFrame();
    const animation = new KeyframeAnimation();
    animation.speed = 100;
    animation.onFrame = this.onFrame;
    animation.play({ start: initialFrame, end: initialFrame });

    this.animation = animation;
  }

  private onFrame = (frameNumber: number) => {
    const frame = this.iconSet[frameNumber];
    if (frame === undefined) {
      log.error('Failed to show tray icon due to the icon being undefined');
    } else {
      this.tray.setImage(frame);
    }
  };

  private loadImages() {
    this.iconSets.regular = this.loadImageSet('');

    switch (process.platform) {
      case 'darwin':
        this.iconSets.template = this.loadImageSet('Template');
        break;
      case 'win32':
        this.iconSets.white = this.loadImageSet('_white');
        this.iconSets.black = this.loadImageSet('_black');
        break;
      case 'linux':
      default:
        this.iconSets.white = this.loadImageSet('_white');
        break;
    }
  }

  private loadImageSet(suffix: string): NativeImage[] {
    const frames = Array.from({ length: 10 }, (_, i) => i + 1);
    return frames.map((frame) => nativeImage.createFromPath(this.getImagePath(frame, suffix)));
  }

  private getImagePath(frame: number, suffix?: string) {
    const basePath = path.resolve(path.join(__dirname, '../../assets/images/menubar icons'));
    const extension = process.platform === 'win32' ? 'ico' : 'png';
    return path.join(basePath, process.platform, `lock-${frame}${suffix}.${extension}`);
  }

  private async getSystemUsesLightTheme(): Promise<boolean | undefined> {
    try {
      // This registry entry contains information about the tray background color. This is
      // needed to decide between white and black icons.
      const { stdout, stderr } = await exec(
        'reg query HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\\ /v SystemUsesLightTheme',
      );

      if (!stderr && stdout) {
        // Split the output into rows
        const rows = stdout.split('\n');
        // Select the row that contains the registry entry result
        const resultRow = rows.find((row) => row.includes('SystemUsesLightTheme'))?.trim();
        // Split the row into words
        const resultRowWords = resultRow?.split(' ').filter((word) => word !== '');
        // Grab value which is last word on the result row
        const value = resultRowWords && resultRowWords[resultRowWords.length - 1];

        if (value) {
          const parsedValue = parseInt(value);
          return parsedValue === 1 ? true : false;
        }
      }

      return undefined;
    } catch (e) {
      const error = e as Error;
      log.error('Failed to read SystemUsesLightTheme,', error.message);
      return undefined;
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
