// @flow

import fs from 'fs';
import log from 'electron-log';
import path from 'path';
import { app } from 'electron';

import type { GuiSettingsState } from '../shared/gui-settings-state';

export default class GuiSettings {
  _state: GuiSettingsState = {
    startMinimized: false,
  };

  load() {
    try {
      const settingsFile = this._filePath();
      const contents = fs.readFileSync(settingsFile, 'utf8');
      const settings = JSON.parse(contents);

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

  get startMinimized(): boolean {
    return this._state.startMinimized;
  }

  _filePath() {
    return path.join(app.getPath('userData'), 'gui_settings.json');
  }
}
