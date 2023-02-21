import { exec } from 'child_process';
import { app, BrowserWindow, dialog, Menu, nativeImage, screen, Tray } from 'electron';
import path from 'path';
import { sprintf } from 'sprintf-js';
import { promisify } from 'util';

import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import { IAccountData, ILocation, TunnelState } from '../shared/daemon-rpc-types';
import { messages, relayLocations } from '../shared/gettext';
import log from '../shared/logging';
import { Scheduler } from '../shared/scheduler';
import { SHOULD_DISABLE_DEVTOOLS_OPEN, SHOULD_FORWARD_RENDERER_LOG } from './command-line-options';
import { DaemonRpc } from './daemon-rpc';
import {
  changeIpcWebContents,
  IpcMainEventChannel,
  unsetIpcWebContents,
} from './ipc-event-channel';
import { WebContentsConsoleInput } from './logging';
import { isMacOs11OrNewer } from './platform-version';
import TrayIconController, { TrayIconType } from './tray-icon-controller';
import WindowController, { WindowControllerDelegate } from './window-controller';

const execAsync = promisify(exec);

export interface UserInterfaceDelegate {
  dismissActiveNotifications(): void;
  updateAccountData(): void;
  connectTunnel(): void;
  reconnectTunnel(): void;
  disconnectTunnel(): void;
  disconnectAndQuit(): void;
  isUnpinnedWindow(): boolean;
  isLoggedIn(): boolean;
  getAccountData(): IAccountData | undefined;
  getTunnelState(): TunnelState;
}

export default class UserInterface implements WindowControllerDelegate {
  private windowController: WindowController;

  private tray: Tray;
  private trayIconController?: TrayIconController;

  // True while file pickers are displayed which is used to decide if the Browser window should be
  // hidden when losing focus.
  private browsingFiles = false;

  private blurNavigationResetScheduler = new Scheduler();
  private backgroundThrottleScheduler = new Scheduler();

  public constructor(
    private delegate: UserInterfaceDelegate,
    private daemonRpc: DaemonRpc,
    private sandboxDisabled: boolean,
    private navigationResetDisabled: boolean,
  ) {
    const window = this.createWindow();

    this.windowController = this.createWindowController(window);
    this.tray = this.createTray();
  }

  public registerIpcListeners() {
    IpcMainEventChannel.app.handleShowOpenDialog(async (options) => {
      this.browsingFiles = true;
      const response = await dialog.showOpenDialog({
        defaultPath: app.getPath('home'),
        ...options,
      });
      this.browsingFiles = false;
      return response;
    });

    IpcMainEventChannel.app.handleShowLaunchDaemonSettings(async () => {
      try {
        await execAsync(
          'open -W x-apple.systempreferences:com.apple.LoginItems-Settings.extension',
        );
      } catch (error) {
        log.error(`Failed to open launch daemon settings: ${error}`);
      }
    });
  }

  public createTrayIconController(
    tunnelState: TunnelState,
    blockWhenDisconnected: boolean,
    monochromaticIcon: boolean,
  ) {
    const iconType = this.trayIconType(tunnelState, blockWhenDisconnected);
    this.trayIconController = new TrayIconController(this.tray, iconType, monochromaticIcon, false);
  }

  public async initializeWindow(isLoggedIn: boolean, tunnelState: TunnelState) {
    if (!this.windowController.window) {
      throw new Error('No window available in initializeWindow');
    }

    const window = this.windowController.window;

    // Make sure the IPC wrapper always has the latest webcontents if any
    window.webContents.on('destroyed', unsetIpcWebContents);
    changeIpcWebContents(window.webContents);

    this.registerWindowListener();
    this.addContextMenu();

    if (process.env.NODE_ENV === 'development') {
      await this.installDevTools();

      if (!SHOULD_DISABLE_DEVTOOLS_OPEN) {
        // The devtools doesn't open on Windows if openDevTools is called without a delay here.
        window.once('ready-to-show', () => window.webContents.openDevTools({ mode: 'detach' }));
      }

      if (SHOULD_FORWARD_RENDERER_LOG) {
        log.addInput(new WebContentsConsoleInput(window.webContents));
      }
    }

    switch (process.platform) {
      case 'win32':
        this.installWindowsMenubarAppWindowHandlers();
        break;
      case 'darwin':
        this.installMacOsMenubarAppWindowHandlers();
        this.setMacOsAppMenu();
        break;
      case 'linux':
        this.setTrayContextMenu(isLoggedIn, tunnelState);
        this.setLinuxAppMenu();
        window.setMenuBarVisibility(false);
        break;
    }

    this.installWindowCloseHandler();
    this.installTrayClickHandlers();

    const filePath = path.resolve(path.join(__dirname, '../renderer/index.html'));
    try {
      await window.loadFile(filePath);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to load index file: ${error.message}`);
    }

    // disable pinch to zoom
    if (this.windowController.webContents) {
      void this.windowController.webContents.setVisualZoomLevelLimits(1, 1);
    }
  }

  public updateTray = (
    isLoggedIn: boolean,
    tunnelState: TunnelState,
    blockWhenDisconnected: boolean,
  ) => {
    this.updateTrayIcon(tunnelState, blockWhenDisconnected);
    this.setTrayContextMenu(isLoggedIn, tunnelState);
    this.setTrayTooltip(tunnelState);
  };

  public async recreateWindow(isLoggedIn: boolean, tunnelState: TunnelState): Promise<void> {
    if (this.tray) {
      this.tray.removeAllListeners();
      // Prevent the IPC webcontents reference to be reset when replacing window. Resetting wouldn't
      // work since the old webContents is destroyed after the IPC wrapper has been updated with the
      // new one.
      this.windowController.webContents?.removeListener('destroyed', unsetIpcWebContents);
      // Remove window close handler that calls `preventDefault` when closed.
      this.windowController.window?.removeListener('close', this.windowCloseHandler);

      const window = this.createWindow();
      changeIpcWebContents(window.webContents);

      this.windowController.close();
      this.windowController = new WindowController(this, window);

      await this.initializeWindow(isLoggedIn, tunnelState);
      this.windowController.show();
    }
  }

  public reloadWindow = () => this.windowController.window?.reload();
  public isWindowVisible = () => this.windowController.isVisible();
  public showWindow = () => this.windowController.show();
  public updateTrayTheme = () => this.trayIconController?.updateTheme() ?? Promise.resolve();
  public setMonochromaticIcon = (value: boolean) =>
    this.trayIconController?.setMonochromaticIcon(value);
  public showNotificationIcon = (value: boolean) =>
    this.trayIconController?.showNotificationIcon(value);
  public setWindowIcon = (icon: string) => this.windowController.window?.setIcon(icon);

  public updateTrayIcon(tunnelState: TunnelState, blockWhenDisconnected: boolean) {
    const type = this.trayIconType(tunnelState, blockWhenDisconnected);
    this.trayIconController?.animateToIcon(type);
  }

  public dispose = () => {
    this.tray.removeAllListeners();
    this.windowController.window?.removeAllListeners();

    // The window is not closable on macOS to be able to hide the titlebar and workaround
    // a shadow bug rendered above the invisible title bar. This also prevents the window from
    // closing normally, even programmatically. Therefore re-enable the close button just before
    // quitting the app.
    // Github issue: https://github.com/electron/electron/issues/15008
    if (process.platform === 'darwin' && this.windowController.window) {
      this.windowController.window.closable = true;
    }

    this.windowController.close();
    this.trayIconController?.dispose();
  };

  private createTray(): Tray {
    const tray = new Tray(nativeImage.createEmpty());
    tray.setToolTip('Mullvad VPN');

    // disable double click on tray icon since it causes weird delay
    tray.setIgnoreDoubleClickEvents(true);

    return tray;
  }

  private createWindow(): BrowserWindow {
    const unpinnedWindow = this.delegate.isUnpinnedWindow();
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
          this.daemonRpc
            .checkVolumes()
            .catch((error) =>
              log.error(`Unable to notify daemon of device event: ${error.message}`),
            );
        });

        appWindow.removeMenu();

        return appWindow;
      }

      default:
        return new BrowserWindow(options);
    }
  }

  private createWindowController(window: BrowserWindow) {
    return new WindowController(this, window);
  }

  private registerWindowListener() {
    this.windowController.window?.on('focus', () => {
      IpcMainEventChannel.window.notifyFocus?.(true);

      this.blurNavigationResetScheduler.cancel();

      // cancel notifications when window appears
      this.delegate.dismissActiveNotifications();

      const accountData = this.delegate.getAccountData();
      if (!accountData || closeToExpiry(accountData.expiry, 4) || hasExpired(accountData.expiry)) {
        this.delegate.updateAccountData();
      }
    });

    this.windowController.window?.on('blur', () => {
      IpcMainEventChannel.window.notifyFocus?.(false);
    });

    // Use hide instead of blur to prevent the navigation reset from happening when bluring an
    // unpinned window.
    this.windowController.window?.on('hide', () => {
      if (process.env.NODE_ENV !== 'development' || !this.navigationResetDisabled) {
        this.blurNavigationResetScheduler.schedule(() => {
          this.windowController.webContents?.setBackgroundThrottling(false);
          IpcMainEventChannel.navigation.notifyReset?.();

          this.backgroundThrottleScheduler.schedule(() => {
            this.windowController.webContents?.setBackgroundThrottling(true);
          }, 1_000);
        }, 120_000);
      }
    });
  }

  private setTrayContextMenu(isLoggedIn: boolean, tunnelState: TunnelState) {
    if (process.platform === 'linux') {
      this.tray.setContextMenu(
        this.createContextMenu(this.daemonRpc.isConnected, isLoggedIn, tunnelState),
      );
    }
  }

  private setTrayTooltip(tunnelState: TunnelState) {
    const tooltip = this.createTooltipText(this.daemonRpc.isConnected, tunnelState);
    this.tray?.setToolTip(tooltip);
  }

  private addContextMenu() {
    const menuTemplate: Electron.MenuItemConstructorOptions[] = [
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      { type: 'separator' },
      { role: 'selectAll' },
    ];

    // add inspect element on right click menu
    this.windowController.window?.webContents.on(
      'context-menu',
      (_e: Event, props: { x: number; y: number; isEditable: boolean }) => {
        const inspectTemplate = [
          {
            label: 'Inspect element',
            click: () => {
              this.windowController.window?.webContents.openDevTools({ mode: 'detach' });
              this.windowController.window?.webContents.inspectElement(props.x, props.y);
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

            Menu.buildFromTemplate(inputMenu).popup({ window: this.windowController.window });
          } else {
            Menu.buildFromTemplate(menuTemplate).popup({ window: this.windowController.window });
          }
        } else if (process.env.NODE_ENV === 'development') {
          // display inspect element for all non-editable
          // elements when in development mode
          Menu.buildFromTemplate(inspectTemplate).popup({ window: this.windowController.window });
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
    const mullvadVpnSubmenu: Electron.MenuItemConstructorOptions[] = [];
    if (process.env.NODE_ENV === 'development') {
      mullvadVpnSubmenu.unshift({ role: 'quit' }, { role: 'reload' }, { role: 'forceReload' });
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

  private installWindowsMenubarAppWindowHandlers() {
    if (this.delegate.isUnpinnedWindow()) {
      return;
    }

    this.windowController.window?.on('blur', () => {
      // Detect if blur happened when user had a cursor above the tray icon.
      const trayBounds = this.tray.getBounds();
      const cursorPos = screen.getCursorScreenPoint();
      const isCursorInside =
        cursorPos.x >= trayBounds.x &&
        cursorPos.y >= trayBounds.y &&
        cursorPos.x <= trayBounds.x + trayBounds.width &&
        cursorPos.y <= trayBounds.y + trayBounds.height;
      if (!isCursorInside && !this.browsingFiles) {
        this.windowController.hide();
      }
    });
  }

  // setup NSEvent monitor to fix inconsistent window.blur on macOS
  // see https://github.com/electron/electron/issues/8689
  private installMacOsMenubarAppWindowHandlers() {
    if (this.delegate.isUnpinnedWindow()) {
      return;
    }

    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const { NSEventMonitor, NSEventMask } = require('nseventmonitor');
    const macEventMonitor = new NSEventMonitor();
    const eventMask = NSEventMask.leftMouseDown | NSEventMask.rightMouseDown;

    this.windowController.window?.on('show', () =>
      macEventMonitor.start(eventMask, () => this.windowController.hide()),
    );
    this.windowController.window?.on('hide', () => macEventMonitor.stop());
    this.windowController.window?.on('blur', () => {
      // Make sure to hide the menubar window when other program captures the focus.
      // But avoid doing that when dev tools capture the focus to make it possible to inspect the UI
      if (
        this.windowController.window?.isVisible() &&
        !this.windowController.window?.webContents.isDevToolsFocused()
      ) {
        this.windowController.hide();
      }
    });
  }

  private installWindowCloseHandler() {
    if (!this.delegate.isUnpinnedWindow()) {
      return;
    }

    this.windowController.window?.on('close', this.windowCloseHandler);
  }

  private windowCloseHandler = (closeEvent: Event) => {
    closeEvent.preventDefault();
    this.windowController.hide();
  };

  private installTrayClickHandlers() {
    switch (process.platform) {
      case 'win32':
        if (this.delegate.isUnpinnedWindow()) {
          // This needs to be executed on click since if it is added to the tray icon it will be
          // displayed on left click as well.
          this.tray?.on('right-click', () =>
            this.popUpContextMenu(this.delegate.isLoggedIn(), this.delegate.getTunnelState()),
          );
          this.tray?.on('click', () => this.windowController.show());
        } else {
          this.tray?.on('right-click', () => this.windowController.hide());
          this.tray?.on('click', () => this.windowController.toggle());
        }
        break;
      case 'darwin':
        this.tray?.on('right-click', () => this.windowController.hide());
        this.tray?.on('click', (event) => {
          if (event.metaKey) {
            setImmediate(() => this.windowController.updatePosition());
          } else {
            if (isMacOs11OrNewer() && !this.windowController.isVisible()) {
              // This is a workaround for this Electron issue, when it's resolved
              // `this.windowController.toggle()` should do the trick on all platforms:
              // https://github.com/electron/electron/issues/28776
              const contextMenu = Menu.buildFromTemplate([]);
              contextMenu.on('menu-will-show', () => this.windowController.show());
              this.tray?.popUpContextMenu(contextMenu);
            } else {
              this.windowController.toggle();
            }
          }
        });
        break;
      case 'linux':
        this.tray?.on('click', () => this.windowController.show());
        break;
    }
  }

  private popUpContextMenu(isLoggedIn: boolean, tunnelState: TunnelState) {
    this.tray.popUpContextMenu(
      this.createContextMenu(this.daemonRpc.isConnected, isLoggedIn, tunnelState),
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
      { type: 'separator' },
      {
        id: 'disconnect',
        label:
          tunnelState.state === 'disconnected'
            ? messages.gettext('Quit')
            : this.escapeContextMenuLabel(messages.gettext('Disconnect & quit')),
        click: this.delegate.disconnectAndQuit,
      },
    ];

    return Menu.buildFromTemplate(template);
  }

  private escapeContextMenuLabel(label: string): string {
    return label.replace('&', '&&');
  }

  private trayIconType(tunnelState: TunnelState, blockWhenDisconnected: boolean): TrayIconType {
    switch (tunnelState.state) {
      case 'connected':
        return 'secured';

      case 'connecting':
        return 'securing';

      case 'error':
        if (!tunnelState.details.blockingError) {
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
  public isUnpinnedWindow = () => this.delegate.isUnpinnedWindow();
  /* eslint-enable @typescript-eslint/member-ordering */
}
