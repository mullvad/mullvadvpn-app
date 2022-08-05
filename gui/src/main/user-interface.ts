import { app, BrowserWindow, Menu, nativeImage, screen, Tray } from 'electron';
import path from 'path';
import { sprintf } from 'sprintf-js';

import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import { IAccountData, ILocation, TunnelState } from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import log from '../shared/logging';
import { Scheduler } from '../shared/scheduler';
import { AppQuitStage } from './index';
import { changeIpcWebContents, IpcMainEventChannel } from './ipc-event-channel';
import { isMacOs11OrNewer } from './platform-version';
import TrayIconController, { TrayIconType } from './tray-icon-controller';
import WindowController, { WindowControllerDelegate } from './window-controller';

export interface UserInterfaceDelegate {
  cancelPendingNotifications(): void;
  resetTunnelStateAnnouncements(): void;
  getUnpinnedWindow(): boolean;
  checkVolumes(): Promise<void>;
  getAppQuitStage(): AppQuitStage;
  updateAccountData(): void;
  isConnectedToDaemon(): boolean;
  isLoggedIn(): boolean;
  getTunnelState(): TunnelState;
  isBrowsingFiles(): boolean;
  getAccountData(): IAccountData | undefined;
  connectTunnel(): void;
  reconnectTunnel(): void;
  disconnectTunnel(): void;
}

export default class UserInterface implements WindowControllerDelegate {
  private windowController: WindowController;

  private tray: Tray;
  private trayIconController?: TrayIconController;

  private blurNavigationResetScheduler = new Scheduler();
  private backgroundThrottleScheduler = new Scheduler();

  public constructor(
    private delegate: UserInterfaceDelegate,
    private sandboxDisabled: boolean,
    private navigationResetDisabled: boolean,
  ) {
    const window = this.createWindow();
    changeIpcWebContents(window.webContents);

    this.windowController = this.createWindowController(window);
    this.tray = this.createTray();
  }

  public createTrayIconController(
    tunnelState: TunnelState,
    blockWhenDisconnected: boolean,
    monochromaticIcon: boolean,
  ) {
    const iconType = this.trayIconType(tunnelState, blockWhenDisconnected);
    this.trayIconController = new TrayIconController(this.tray, iconType, monochromaticIcon);
  }

  public async initializeWindow() {
    if (!this.windowController?.window) {
      throw new Error('No windowController or window available in initializeWindow');
    }

    const windowController = this.windowController;
    const window = this.windowController.window;

    this.registerWindowListener(windowController, this.delegate.getAccountData());
    this.addContextMenu(windowController);

    if (process.env.NODE_ENV === 'development') {
      await this.installDevTools();

      // The devtools doesn't open on Windows if openDevTools is called without a delay here.
      window.once('ready-to-show', () => window.webContents.openDevTools({ mode: 'detach' }));
    }

    switch (process.platform) {
      case 'win32':
        this.installWindowsMenubarAppWindowHandlers(
          this.windowController,
          this.delegate.isBrowsingFiles(),
        );
        break;
      case 'darwin':
        this.installMacOsMenubarAppWindowHandlers(this.windowController);
        this.setMacOsAppMenu();
        break;
      case 'linux':
        this.setTrayContextMenu();
        this.setLinuxAppMenu();
        this.windowController.window.setMenuBarVisibility(false);
        break;
    }

    this.installWindowCloseHandler(this.windowController);
    this.installTrayClickHandlers();

    const filePath = path.resolve(path.join(__dirname, '../renderer/index.html'));
    try {
      await this.windowController.window.loadFile(filePath);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to load index file: ${error.message}`);
    }

    // disable pinch to zoom
    if (this.windowController.webContents) {
      void this.windowController.webContents.setVisualZoomLevelLimits(1, 1);
    }
  }

  public setTrayContextMenu = () => {
    if (process.platform === 'linux') {
      this.tray.setContextMenu(
        this.createContextMenu(
          this.delegate.isConnectedToDaemon(),
          this.delegate.isLoggedIn(),
          this.delegate.getTunnelState(),
        ),
      );
    }
  };

  public setTrayTooltip = () => {
    const tooltip = this.createTooltipText(
      this.delegate.isConnectedToDaemon(),
      this.delegate.getTunnelState(),
    );
    this.tray?.setToolTip(tooltip);
  };

  public async recreateWindow(): Promise<void> {
    if (this.tray && this.windowController) {
      this.tray.removeAllListeners();

      const window = this.createWindow();
      changeIpcWebContents(window.webContents);

      this.windowController.close();
      this.windowController = new WindowController(this, window);

      await this.initializeWindow();
      this.windowController.show();
    }
  }

  public reloadWindow = () => this.windowController.window?.reload();
  public isWindowVisible = () => this.windowController.isVisible();
  public showWindow = () => this.windowController.show();
  public updateTrayTheme = () => this.trayIconController?.updateTheme();
  public setUseMonochromaticTrayIcon = (value: boolean) =>
    this.trayIconController?.setUseMonochromaticIcon(value);
  public animateTrayToIcon = (type: TrayIconType) => this.trayIconController?.animateToIcon(type);
  public setWindowIcon = (icon: string) => this.windowController.window?.setIcon(icon);

  public setWindowClosable = (value: boolean) => {
    if (this.windowController.window) {
      this.windowController.window.closable = value;
    }
  };

  public updateTrayIcon(tunnelState: TunnelState, blockWhenDisconnected: boolean) {
    const type = this.trayIconType(tunnelState, blockWhenDisconnected);
    this.animateTrayToIcon(type);
  }

  public dispose = () => this.trayIconController?.dispose();

  private createTray(): Tray {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setToolTip('Mullvad VPN');

    // disable double click on tray icon since it causes weird delay
    tray.setIgnoreDoubleClickEvents(true);

    return tray;
  }

  private createWindow(): BrowserWindow {
    const unpinnedWindow = this.delegate.getUnpinnedWindow();
    const { width, height } = WindowController.getContentSize(unpinnedWindow);

    const options: Electron.BrowserWindowConstructorOptions = {
      useContentSize: true,
      width,
      height,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      show: false,
      frame: unpinnedWindow,
      webPreferences: {
        preload: path.join(__dirname, '../renderer/preloadBundle.js'),
        nodeIntegration: false,
        nodeIntegrationInWorker: false,
        nodeIntegrationInSubFrames: false,
        sandbox: !this.sandboxDisabled,
        contextIsolation: true,
        spellcheck: false,
        devTools: process.env.NODE_ENV === 'development',
      },
    };

    switch (process.platform) {
      case 'darwin': {
        // setup window flags to mimic popover on macOS
        const appWindow = new BrowserWindow({
          ...options,
          titleBarStyle: unpinnedWindow ? 'default' : 'customButtonsOnHover',
          minimizable: unpinnedWindow,
          closable: unpinnedWindow,
          transparent: !unpinnedWindow,
        });

        // make the window visible on all workspaces and prevent the icon from showing in the dock
        // and app switcher.
        if (unpinnedWindow) {
          void app.dock.show();
        } else {
          appWindow.setVisibleOnAllWorkspaces(true);
          app.dock.hide();
        }

        return appWindow;
      }

      case 'win32': {
        // setup window flags to mimic an overlay window
        const appWindow = new BrowserWindow({
          ...options,
          // Due to a bug in Electron the app is sometimes placed behind other apps when opened.
          // Setting alwaysOnTop to true ensures that the app is placed on top. Electron issue:
          // https://github.com/electron/electron/issues/25915
          alwaysOnTop: !unpinnedWindow,
          skipTaskbar: !unpinnedWindow,
          // Workaround for sub-pixel anti-aliasing
          // https://github.com/electron/electron/blob/main/docs/faq.md#the-font-looks-blurry-what-is-this-and-what-can-i-do
          backgroundColor: '#fff',
        });
        const WM_DEVICECHANGE = 0x0219;
        const DBT_DEVICEARRIVAL = 0x8000;
        const DBT_DEVICEREMOVECOMPLETE = 0x8004;
        appWindow.hookWindowMessage(WM_DEVICECHANGE, (wParam) => {
          const wParamL = wParam.readBigInt64LE(0);
          if (wParamL != DBT_DEVICEARRIVAL && wParamL != DBT_DEVICEREMOVECOMPLETE) {
            return;
          }
          this.delegate
            .checkVolumes()
            .catch((error) =>
              log.error(`Unable to notify daemon of device event: ${error.message}`),
            );
        });

        appWindow.removeMenu();

        return appWindow;
      }

      case 'linux':
        return new BrowserWindow({
          ...options,
        });

      default: {
        return new BrowserWindow(options);
      }
    }
  }

  private createWindowController(window: BrowserWindow) {
    return new WindowController(this, window);
  }

  private registerWindowListener(windowController: WindowController, accountData?: IAccountData) {
    windowController.window?.on('focus', () => {
      IpcMainEventChannel.window.notifyFocus?.(true);

      this.blurNavigationResetScheduler.cancel();

      // cancel notifications when window appears
      this.delegate.cancelPendingNotifications();

      if (!accountData || closeToExpiry(accountData.expiry, 4) || hasExpired(accountData.expiry)) {
        this.delegate.updateAccountData();
      }
    });

    windowController.window?.on('blur', () => {
      IpcMainEventChannel.window.notifyFocus?.(false);

      // ensure notification guard is reset
      this.delegate.resetTunnelStateAnnouncements();
    });

    // Use hide instead of blur to prevent the navigation reset from happening when bluring an
    // unpinned window.
    windowController.window?.on('hide', () => {
      if (process.env.NODE_ENV !== 'development' || !this.navigationResetDisabled) {
        this.blurNavigationResetScheduler.schedule(() => {
          windowController.webContents?.setBackgroundThrottling(false);
          IpcMainEventChannel.navigation.notifyReset?.();

          this.backgroundThrottleScheduler.schedule(() => {
            windowController.webContents?.setBackgroundThrottling(true);
          }, 1_000);
        }, 120_000);
      }
    });
  }

  private addContextMenu(windowController: WindowController) {
    const menuTemplate: Electron.MenuItemConstructorOptions[] = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectAll' },
    ];

    // add inspect element on right click menu
    windowController.window?.webContents.on(
      'context-menu',
      (_e: Event, props: { x: number; y: number; isEditable: boolean }) => {
        const inspectTemplate = [
          {
            label: 'Inspect element',
            click() {
              windowController.window?.webContents.openDevTools({ mode: 'detach' });
              windowController.window?.webContents.inspectElement(props.x, props.y);
            },
          },
        ];

        if (props.isEditable) {
          // mixin 'inspect element' into standard menu when in development mode
          if (process.env.NODE_ENV === 'development') {
            const inputMenu: Electron.MenuItemConstructorOptions[] = [
              { type: 'separator' },
              ...inspectTemplate,
            ];

            Menu.buildFromTemplate(inputMenu).popup({ window: windowController.window });
          } else {
            Menu.buildFromTemplate(menuTemplate).popup({ window: windowController.window });
          }
        } else if (process.env.NODE_ENV === 'development') {
          // display inspect element for all non-editable
          // elements when in development mode
          Menu.buildFromTemplate(inspectTemplate).popup({ window: windowController.window });
        }
      },
    );
  }

  private async installDevTools() {
    const { default: installer, REACT_DEVELOPER_TOOLS, REDUX_DEVTOOLS } = await import(
      'electron-devtools-installer'
    );
    const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
    const options = { forceDownload, loadExtensionOptions: { allowFileAccess: true } };
    try {
      await installer(REACT_DEVELOPER_TOOLS, options);
      await installer(REDUX_DEVTOOLS, options);
    } catch (e) {
      const error = e as Error;
      log.info(`Error installing extension: ${error.message}`);
    }
  }

  // On macOS, hotkeys are bound to the app menu and won't work if it's not set,
  // even though the app menu itself is not visible because the app does not appear in the dock.
  private setMacOsAppMenu() {
    const mullvadVpnSubmenu: Electron.MenuItemConstructorOptions[] = [{ role: 'quit' }];
    if (process.env.NODE_ENV === 'development') {
      mullvadVpnSubmenu.unshift({ role: 'reload' }, { role: 'forceReload' });
    }

    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Mullvad VPN',
        submenu: mullvadVpnSubmenu,
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
          { type: 'separator' },
          { role: 'selectAll' },
        ],
      },
    ];
    Menu.setApplicationMenu(Menu.buildFromTemplate(template));
  }

  private setLinuxAppMenu() {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Mullvad VPN',
        submenu: [{ role: 'quit' }],
      },
    ];
    Menu.setApplicationMenu(Menu.buildFromTemplate(template));
  }

  private installWindowsMenubarAppWindowHandlers(
    windowController: WindowController,
    browsingFiles: boolean,
  ) {
    if (!this.delegate.getUnpinnedWindow()) {
      windowController.window?.on('blur', () => {
        // Detect if blur happened when user had a cursor above the tray icon.
        const trayBounds = this.tray.getBounds();
        const cursorPos = screen.getCursorScreenPoint();
        const isCursorInside =
          cursorPos.x >= trayBounds.x &&
          cursorPos.y >= trayBounds.y &&
          cursorPos.x <= trayBounds.x + trayBounds.width &&
          cursorPos.y <= trayBounds.y + trayBounds.height;
        if (!isCursorInside && !browsingFiles) {
          windowController.hide();
        }
      });
    }
  }

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  private installMacOsMenubarAppWindowHandlers(windowController: WindowController) {
    if (!this.delegate.getUnpinnedWindow()) {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
      const macEventMonitor = new NSEventMonitor();
      const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;

      windowController.window?.on('show', () =>
        macEventMonitor.start(eventMask, () => windowController.hide()),
      );
      windowController.window?.on('hide', () => macEventMonitor.stop());
      windowController.window?.on('blur', () => {
        // Make sure to hide the menubar window when other program captures the focus.
        // But avoid doing that when dev tools capture the focus to make it possible to inspect the UI
        if (
          windowController.window?.isVisible() &&
          !windowController.window?.webContents.isDevToolsFocused()
        ) {
          windowController.hide();
        }
      });
    }
  }

  private installWindowCloseHandler(windowController: WindowController) {
    if (this.delegate.getUnpinnedWindow()) {
      windowController.window?.on('close', (closeEvent: Event) => {
        if (this.delegate.getAppQuitStage() !== AppQuitStage.ready) {
          closeEvent.preventDefault();
          windowController.hide();
        }
      });
    }
  }

  private installTrayClickHandlers() {
    switch (process.platform) {
      case 'win32':
        if (this.delegate.getUnpinnedWindow()) {
          // This needs to be executed on click since if it is added to the tray icon it will be
          // displayed on left click as well.
          this.tray?.on('right-click', () => this.popUpContextMenu());
          this.tray?.on('click', () => this.windowController?.show());
        } else {
          this.tray?.on('right-click', () => this.windowController?.hide());
          this.tray?.on('click', () => this.windowController?.toggle());
        }
        break;
      case 'darwin':
        this.tray?.on('right-click', () => this.windowController?.hide());
        this.tray?.on('click', (event) => {
          if (event.metaKey) {
            setImmediate(() => this.windowController?.updatePosition());
          } else {
            if (isMacOs11OrNewer() && !this.windowController?.isVisible()) {
              // This is a workaround for this Electron issue, when it's resolved
              // `this.windowController?.toggle()` should do the trick on all platforms:
              // https://github.com/electron/electron/issues/28776
              const contextMenu = Menu.buildFromTemplate([]);
              contextMenu.on('menu-will-show', () => this.windowController?.show());
              this.tray?.popUpContextMenu(contextMenu);
            } else {
              this.windowController?.toggle();
            }
          }
        });
        break;
      case 'linux':
        this.tray?.on('click', () => this.windowController?.show());
        break;
    }
  }

  private popUpContextMenu() {
    this.tray.popUpContextMenu(
      this.createContextMenu(
        this.delegate.isConnectedToDaemon(),
        this.delegate.isLoggedIn(),
        this.delegate.getTunnelState(),
      ),
    );
  }

  private createTooltipText(connectedToDaemon: boolean, tunnelState: TunnelState): string {
    if (!connectedToDaemon) {
      return messages.pgettext('tray-icon-context-menu', 'Disconnected from system service');
    }

    switch (tunnelState.state) {
      case 'disconnected':
        return messages.gettext('Disconnected');
      case 'disconnecting':
        return messages.gettext('Disconnecting');
      case 'connecting': {
        const location = this.createLocationString(tunnelState.details?.location);
        return location
          ? sprintf(messages.pgettext('tray-icon-tooltip', 'Connecting. %(location)s'), {
              location,
            })
          : messages.gettext('Connecting');
      }
      case 'connected': {
        const location = this.createLocationString(tunnelState.details.location);
        return location
          ? sprintf(messages.pgettext('tray-icon-tooltip', 'Connected. %(location)s'), {
              location,
            })
          : messages.gettext('Connected');
      }
    }

    return 'Mullvad VPN';
  }

  private createLocationString(location?: ILocation): string | undefined {
    if (location === undefined) {
      return undefined;
    }

    const country = relayLocations.gettext(location.country);
    return location.city
      ? sprintf(messages.pgettext('tray-icon-tooltip', '%(city)s, %(country)s'), {
          city: relayLocations.gettext(location.city),
          country,
        })
      : country;
  }

  private createContextMenu(
    connectedToDaemon: boolean,
    loggedIn: boolean,
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
        enabled: connectEnabled(connectedToDaemon, loggedIn, tunnelState.state),
        click: this.delegate.connectTunnel,
      },
      {
        id: 'reconnect',
        label: messages.gettext('Reconnect'),
        enabled: reconnectEnabled(connectedToDaemon, loggedIn, tunnelState.state),
        click: this.delegate.reconnectTunnel,
      },
      {
        id: 'disconnect',
        label: messages.gettext('Disconnect'),
        enabled: disconnectEnabled(connectedToDaemon, tunnelState.state),
        click: this.delegate.disconnectTunnel,
      },
    ];

    return Menu.buildFromTemplate(template);
  }

  private trayIconType(tunnelState: TunnelState, blockWhenDisconnected: boolean): TrayIconType {
    switch (tunnelState.state) {
      case 'connected':
        return 'secured';

      case 'connecting':
        return 'securing';

      case 'error':
        if (!tunnelState.details.blockFailure) {
          return 'securing';
        } else {
          return 'unsecured';
        }
      case 'disconnecting':
        return 'securing';

      case 'disconnected':
        if (blockWhenDisconnected) {
          return 'securing';
        } else {
          return 'unsecured';
        }
    }
  }

  /* eslint-disable @typescript-eslint/member-ordering */
  // WindowControllerDelegate
  public getTrayBounds = () => this.tray.getBounds();
  public getUnpinnedWindow = this.delegate.getUnpinnedWindow;
  /* eslint-enable @typescript-eslint/member-ordering */
}
