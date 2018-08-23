// @flow

import { screen } from 'electron';
import type { BrowserWindow, Tray, Display } from 'electron';

type Position = { x: number, y: number };

interface WindowPositioning {
  getPosition(window: BrowserWindow): Position;
}

class StandaloneWindowPositioning implements WindowPositioning {
  getPosition(window: BrowserWindow): Position {
    const windowBounds = window.getBounds();

    const primaryDisplay = screen.getPrimaryDisplay();
    const workArea = primaryDisplay.workArea;
    const maxX = workArea.x + workArea.width - windowBounds.width;
    const maxY = workArea.y + workArea.height - windowBounds.height;

    const x = Math.min(Math.max(windowBounds.x, workArea.x), maxX);
    const y = Math.min(Math.max(windowBounds.y, workArea.y), maxY);

    return { x, y };
  }
}

class AttachedToTrayWindowPositioning implements WindowPositioning {
  _tray: Tray;

  constructor(tray: Tray) {
    this._tray = tray;
  }

  getPosition(window: BrowserWindow): Position {
    const windowBounds = window.getBounds();
    const trayBounds = this._tray.getBounds();

    const primaryDisplay = screen.getPrimaryDisplay();
    const workArea = primaryDisplay.workArea;
    const placement = this._getTrayPlacement();
    const maxX = workArea.x + workArea.width - windowBounds.width;
    const maxY = workArea.y + workArea.height - windowBounds.height;

    let x = 0,
      y = 0;
    switch (placement) {
      case 'top':
        x = trayBounds.x + (trayBounds.width - windowBounds.width) * 0.5;
        y = workArea.y;
        break;

      case 'bottom':
        x = trayBounds.x + (trayBounds.width - windowBounds.width) * 0.5;
        y = workArea.y + workArea.height - windowBounds.height;
        break;

      case 'left':
        x = workArea.x;
        y = trayBounds.y + (trayBounds.height - windowBounds.height) * 0.5;
        break;

      case 'right':
        x = workArea.width - windowBounds.width;
        y = trayBounds.y + (trayBounds.height - windowBounds.height) * 0.5;
        break;

      case 'none':
        x = workArea.x + (workArea.width - windowBounds.width) * 0.5;
        y = workArea.y + (workArea.height - windowBounds.height) * 0.5;
        break;
    }

    x = Math.min(Math.max(x, workArea.x), maxX);
    y = Math.min(Math.max(y, workArea.y), maxY);

    return { x, y };
  }

  _getTrayPlacement() {
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
}

export default class WindowController {
  _window: BrowserWindow;
  _windowPositioning: WindowPositioning;
  _isWindowReady = false;

  get window(): BrowserWindow {
    return this._window;
  }

  constructor(window: BrowserWindow, tray: Tray) {
    this._window = window;

    if (process.platform === 'linux') {
      this._windowPositioning = new StandaloneWindowPositioning();
    } else {
      this._windowPositioning = new AttachedToTrayWindowPositioning(tray);
    }

    this._installDisplayMetricsHandler();
    this._installWindowReadyHandlers();
  }

  show(whenReady: boolean = true) {
    if (whenReady) {
      this._executeWhenWindowIsReady(() => this._showImmediately());
    } else {
      this._showImmediately();
    }
  }

  hide() {
    this._window.hide();
  }

  toggle() {
    if (this._window.isVisible()) {
      this.hide();
    } else {
      this.show();
    }
  }

  _showImmediately() {
    const window = this._window;

    this._updatePosition();

    window.show();
    window.focus();
  }

  _updatePosition() {
    const { x, y } = this._windowPositioning.getPosition(this._window);
    this._window.setPosition(x, y, false);
  }

  // Installs display event handlers to update the window position on any changes in the display or workarea dimensions.
  _installDisplayMetricsHandler() {
    screen.addListener('display-metrics-changed', this._onDisplayMetricsChanged);
    this._window.once('closed', () => {
      screen.removeListener('display-metrics-changed', this._onDisplayMetricsChanged);
    });
  }

  _onDisplayMetricsChanged = (_event: any, _display: Display, changedMetrics: Array<string>) => {
    if (changedMetrics.includes('workArea') && this._window.isVisible()) {
      this._updatePosition();
    }
  };

  _installWindowReadyHandlers() {
    this._window.once('ready-to-show', () => {
      this._isWindowReady = true;
    });
  }

  _executeWhenWindowIsReady(closure: () => any) {
    if (this._isWindowReady) {
      closure();
    } else {
      this._window.once('ready-to-show', () => {
        closure();
      });
    }
  }
}
