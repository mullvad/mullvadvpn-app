import { BrowserWindow, Display, screen, Tray, WebContents } from 'electron';

import { IWindowShapeParameters } from '../shared/ipc-types';
import { Scheduler } from '../shared/scheduler';
import { IpcMainEventChannel } from './ipc-event-channel';
import { isWindows11OrNewer } from './platform-version';

interface IPosition {
  x: number;
  y: number;
}

interface IWindowPositioning {
  getPosition(window: BrowserWindow): IPosition;
  getWindowShapeParameters(window: BrowserWindow): IWindowShapeParameters;
}

export interface WindowControllerDelegate {
  getTrayBounds: Tray['getBounds'];
  isUnpinnedWindow(): boolean;
}

class StandaloneWindowPositioning implements IWindowPositioning {
  public getPosition(window: BrowserWindow): IPosition {
    const windowBounds = window.getBounds();

    const primaryDisplay = screen.getPrimaryDisplay();
    const workArea = primaryDisplay.workArea;
    const maxX = workArea.x + workArea.width - windowBounds.width;
    const maxY = workArea.y + workArea.height - windowBounds.height;

    const x = Math.min(Math.max(windowBounds.x, workArea.x), maxX);
    const y = Math.min(Math.max(windowBounds.y, workArea.y), maxY);

    return { x, y };
  }

  public getWindowShapeParameters(_window: BrowserWindow): IWindowShapeParameters {
    return {};
  }
}

class AttachedToTrayWindowPositioning implements IWindowPositioning {
  constructor(private delegate: WindowControllerDelegate) {}

  public getPosition(window: BrowserWindow): IPosition {
    const windowBounds = window.getBounds();
    const trayBounds = this.delegate.getTrayBounds();

    const activeDisplay = screen.getDisplayNearestPoint({
      x: trayBounds.x,
      y: trayBounds.y,
    });
    const workArea = activeDisplay.workArea;
    const placement = this.getTrayPlacement();
    const maxX = workArea.x + workArea.width - windowBounds.width;
    const maxY = workArea.y + workArea.height - windowBounds.height;

    const margin = this.getWindowMargin();

    let x = 0;
    let y = 0;

    switch (placement) {
      case 'top':
        x = trayBounds.x + (trayBounds.width - windowBounds.width) * 0.5;
        y = workArea.y + margin;
        break;

      case 'bottom':
        x = trayBounds.x + (trayBounds.width - windowBounds.width) * 0.5;
        y = workArea.y + workArea.height - windowBounds.height - margin;
        break;

      case 'left':
        x = workArea.x + margin;
        y = trayBounds.y + (trayBounds.height - windowBounds.height) * 0.5;
        break;

      case 'right':
        x = workArea.width - windowBounds.width - margin;
        y = trayBounds.y + (trayBounds.height - windowBounds.height) * 0.5;
        break;

      case 'none':
        x = workArea.x + (workArea.width - windowBounds.width) * 0.5;
        y = workArea.y + (workArea.height - windowBounds.height) * 0.5;
        break;
    }

    x = Math.min(Math.max(x, workArea.x), maxX);
    y = Math.min(Math.max(y, workArea.y), maxY);

    return {
      x: Math.round(x),
      y: Math.round(y),
    };
  }

  public getWindowShapeParameters(window: BrowserWindow): IWindowShapeParameters {
    const trayBounds = this.delegate.getTrayBounds();
    const windowBounds = window.getBounds();
    const arrowPosition = trayBounds.x - windowBounds.x + trayBounds.width * 0.5;
    return {
      arrowPosition,
    };
  }

  private getTrayPlacement() {
    switch (process.platform) {
      case 'darwin':
        // macOS has menubar always placed at the top
        return 'top';

      case 'win32': {
        // taskbar occupies some part of the screen excluded from work area
        const primaryDisplay = screen.getPrimaryDisplay();
        const displaySize = primaryDisplay.size;
        const workArea = primaryDisplay.workArea;

        if (workArea.width < displaySize.width) {
          return workArea.x > 0 ? 'left' : 'right';
        } else if (workArea.height < displaySize.height) {
          return workArea.y > 0 ? 'top' : 'bottom';
        } else {
          return 'none';
        }
      }

      default:
        return 'none';
    }
  }

  private getWindowMargin() {
    if (isWindows11OrNewer()) {
      // Tray applications are positioned aproximately 10px from the tray in Windows 11.
      return 10;
    } else if (process.platform === 'darwin') {
      return 5;
    } else {
      return 0;
    }
  }
}

export default class WindowController {
  private windowValue: BrowserWindow;
  private webContentsValue: WebContents;
  private windowPositioning: IWindowPositioning;

  private windowPositioningScheduler = new Scheduler();

  get window(): BrowserWindow | undefined {
    return this.windowValue.isDestroyed() ? undefined : this.windowValue;
  }

  get webContents(): WebContents | undefined {
    return this.webContentsValue.isDestroyed() ? undefined : this.webContentsValue;
  }

  constructor(private delegate: WindowControllerDelegate, windowValue: BrowserWindow) {
    this.windowValue = windowValue;
    this.webContentsValue = windowValue.webContents;
    this.windowPositioning = delegate.isUnpinnedWindow()
      ? new StandaloneWindowPositioning()
      : new AttachedToTrayWindowPositioning(delegate);

    this.installDisplayMetricsHandler();
    this.installHideHandler();
  }

  public show(whenReady = true) {
    if (whenReady) {
      this.executeWhenWindowIsReady(() => this.showImmediately());
    } else {
      this.showImmediately();
    }
  }

  public hide() {
    this.window?.hide();
  }

  public toggle() {
    if (this.window?.isVisible()) {
      this.hide();
    } else {
      this.show();
    }
  }

  public isVisible(): boolean {
    return this.window !== undefined && this.window.isVisible() && !this.window.isMinimized();
  }

  public updatePosition() {
    if (this.window) {
      const position = this.windowPositioning.getPosition(this.window);
      const size = WindowController.getContentSize(this.delegate.isUnpinnedWindow());
      this.window.setBounds({ ...position, ...size }, false);
    }

    this.notifyUpdateWindowShape();
  }

  public close() {
    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.closeDevTools();
      this.window.closable = true;
      this.window.close();
    }
  }

  public static getContentSize(unpinnedWindow: boolean): { width: number; height: number } {
    return {
      width: 320,
      height: WindowController.getContentHeight(unpinnedWindow),
    };
  }

  private installHideHandler() {
    this.window?.addListener('hide', () => this.windowPositioningScheduler.cancel());
    this.window?.addListener('closed', () => this.windowPositioningScheduler.cancel());
  }

  private showImmediately() {
    const window = this.window;

    // When running with unpinned window on Windows there's a bug that causes the app to become
    // wider if opened from minimized if the updated position is set before the window is opened.
    // Unfortunately the order can't always be changed since this would cause the Window to "jump"
    // in other scenarios.
    if (
      process.platform === 'win32' &&
      this.windowPositioning instanceof StandaloneWindowPositioning
    ) {
      window?.show();
      window?.focus();

      this.updatePosition();
    } else {
      this.updatePosition();

      window?.show();
      window?.focus();
    }
  }

  private notifyUpdateWindowShape() {
    if (this.window) {
      const shapeParameters = this.windowPositioning.getWindowShapeParameters(this.window);

      IpcMainEventChannel.window.notifyShape?.(shapeParameters);
    }
  }

  // Installs display event handlers to update the window position on any changes in the display or
  // workarea dimensions.
  private installDisplayMetricsHandler() {
    if (this.window) {
      screen.addListener('display-metrics-changed', this.onDisplayMetricsChanged);
      this.window.once('closed', () => {
        screen.removeListener('display-metrics-changed', this.onDisplayMetricsChanged);
      });
    }
  }

  private onDisplayMetricsChanged = (
    _event: Electron.Event,
    _display: Display,
    changedMetrics: string[],
  ) => {
    if (changedMetrics.includes('workArea') && this.window?.isVisible()) {
      this.onWorkAreaSizeChange();
      if (process.platform === 'win32') {
        this.windowPositioningScheduler.schedule(() => this.onWorkAreaSizeChange(), 500);
      }
    }

    // On Linux and Windows, the window won't be properly rescaled back to it's original
    // size if the DPI scaling factor is changed.
    // https://github.com/electron/electron/issues/11050
    if (
      changedMetrics.includes('scaleFactor') &&
      (process.platform === 'win32' || process.platform === 'linux')
    ) {
      this.forceResizeWindow();
    }
  };

  private onWorkAreaSizeChange() {
    this.updatePosition();
  }

  private forceResizeWindow() {
    const { width, height } = WindowController.getContentSize(this.delegate.isUnpinnedWindow());
    this.window?.setContentSize(width, height);
  }

  private executeWhenWindowIsReady(closure: () => void) {
    if (this.webContents?.isLoading() === false && this.webContents?.getURL() !== '') {
      closure();
    } else {
      this.webContents?.once('did-stop-loading', () => {
        closure();
      });
    }
  }

  // On both Linux and Windows the app height is applied incorrectly:
  // https://github.com/electron/electron/issues/28777
  private static getContentHeight(unpinnedWindow: boolean): number {
    // The height we want to achieve.
    const contentHeight = 568;

    switch (process.platform) {
      case 'win32':
        // On Windows the app height ends up slightly lower than we set it to if running in unpinned
        // mode and the app becomes a tiny bit taller when pinned to task bar.
        return unpinnedWindow ? contentHeight + 25 : contentHeight;
      default:
        return contentHeight;
    }
  }
}
