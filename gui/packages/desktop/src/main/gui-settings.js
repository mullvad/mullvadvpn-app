// @flow

import fs from 'fs';
import log from 'electron-log';
import path from 'path';
import { app } from 'electron';

import IpcEventChannel from '../shared/ipc-event-channel';
import type { GuiSettingsState } from '../shared/gui-settings-state';

export default class GuiSettings {
  _state: GuiSettingsState = {
    monochromaticIcon: false,
    startMinimized: false,
  };

  _notify: ?(GuiSettingsState) => void;

  load() {
    try {
      const settingsFile = this._filePath();
      const contents = fs.readFileSync(settingsFile, 'utf8');
      const settings = JSON.parse(contents);

      this._state.monochromaticIcon = settings.monochromaticIcon || false;
      this._state.startMinimized = settings.startMinimized || false;
    } catch (error) {
      log.error(`Failed to read GUI settings file: ${error}`);
    }
  }

  store() {
    try {
      const settingsFile = this._filePath();

      fs.writeFileSync(settingsFile, JSON.stringify(this._state));
    } catch (error) {
      log.error(`Failed to write GUI settings file: ${error}`);
    }
  }

  get state(): GuiSettingsState {
    return this._state;
  }

  get monochromaticIcon(): boolean {
    return this._state.monochromaticIcon;
  }

  get startMinimized(): boolean {
    return this._state.startMinimized;
  }

  registerIpcHandlers(ipcEventChannel: IpcEventChannel) {
    this._notify = ipcEventChannel.guiSettings.notify;

    ipcEventChannel.guiSettings.handleMonochromaticIcon((monochromaticIcon: boolean) => {
      this._state.monochromaticIcon = monochromaticIcon;
      this._settingsChanged();
    });
    ipcEventChannel.guiSettings.handleStartMinimized((startMinimized: boolean) => {
      this._state.startMinimized = startMinimized;
      this._settingsChanged();
    });
  }

  _filePath() {
    return path.join(app.getPath('userData'), 'gui_settings.json');
  }

  _settingsChanged() {
    this.store();

    if (this._notify) {
      this._notify(this._state);
    }
  }
}
