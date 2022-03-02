import { exec as execAsync } from 'child_process';
import { Menu, nativeImage, NativeImage, Tray } from 'electron';
import path from 'path';
import { sprintf } from 'sprintf-js';
import { promisify } from 'util';
import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import { AccountToken, TunnelState } from '../shared/daemon-rpc-types';
import { messages } from '../shared/gettext';
import log from '../shared/logging';
import KeyframeAnimation from './keyframe-animation';
import WindowController from './window-controller';

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
    private windowController: WindowController,
    private iconTypeValue: TrayIconType,
    private useMonochromaticIconValue: boolean,
    private connect: () => void,
    private reconnect: () => void,
    private disconnect: () => void,
  ) {
    this.loadImages();
  }

  public dispose() {
    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }
  }

  public setWindowController(windowController: WindowController) {
    this.windowController = windowController;
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

  public setContextMenu(
    connectedToDaemon: boolean,
    accountToken: AccountToken | undefined,
    tunnelState: TunnelState,
  ) {
    if (process.platform === 'linux') {
      this.tray.setContextMenu(
        this.createContextMenu(connectedToDaemon, accountToken, tunnelState),
      );
    }
  }

  public popUpContextMenu(
    connectedToDaemon: boolean,
    accountToken: AccountToken | undefined,
    tunnelState: TunnelState,
  ) {
    this.tray.popUpContextMenu(
      this.createContextMenu(connectedToDaemon, accountToken, tunnelState),
    );
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

  private createContextMenu(
    connectedToDaemon: boolean,
    accountToken: AccountToken | undefined,
    tunnelState: TunnelState,
  ) {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: sprintf(messages.pgettext('tray-icon-context-menu', 'Open %(mullvadVpn)s'), {
          mullvadVpn: 'Mullvad VPN',
        }),
        click: () => this.windowController.show(),
      },
      { type: 'separator' },
      {
        id: 'connect',
        label: messages.gettext('Connect'),
        enabled: connectEnabled(connectedToDaemon, accountToken, tunnelState.state),
        click: this.connect,
      },
      {
        id: 'reconnect',
        label: messages.gettext('Reconnect'),
        enabled: reconnectEnabled(connectedToDaemon, accountToken, tunnelState.state),
        click: this.reconnect,
      },
      {
        id: 'disconnect',
        label: messages.gettext('Disconnect'),
        enabled: disconnectEnabled(connectedToDaemon, tunnelState.state),
        click: this.disconnect,
      },
    ];

    return Menu.buildFromTemplate(template);
  }
}
