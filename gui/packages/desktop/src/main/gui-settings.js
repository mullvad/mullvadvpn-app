// @flow

import fs from 'fs';
import log from 'electron-log';
import path from 'path';
import { app } from 'electron';

import type { GuiSettingsState } from '../shared/gui-settings-state';

export default class GuiSettings {
  _state: GuiSettingsState = {
    autoConnect: true,
    monochromaticIcon: false,
    startMinimized: false,
  };

  onChange: ?(newState: GuiSettingsState, oldState: GuiSettingsState) => void;

  load() {
    try {
      const settingsFile = this._filePath();
      const contents = fs.readFileSync(settingsFile, 'utf8');
      const settings = JSON.parse(contents);

      this._state.autoConnect =
        typeof settings.autoConnect === 'boolean' ? settings.autoConnect : true;
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

  set autoConnect(newValue: boolean) {
    this._changeStateAndNotify({ ...this._state, autoConnect: newValue });
  }

  get autoConnect(): boolean {
    return this._state.autoConnect;
  }

  set monochromaticIcon(newValue: boolean) {
    this._changeStateAndNotify({ ...this._state, monochromaticIcon: newValue });
  }

  get monochromaticIcon(): boolean {
    return this._state.monochromaticIcon;
  }

  set startMinimized(newValue: boolean) {
    this._changeStateAndNotify({ ...this._state, startMinimized: newValue });
  }

  get startMinimized(): boolean {
    return this._state.startMinimized;
  }

  _filePath() {
    return path.join(app.getPath('userData'), 'gui_settings.json');
  }

  _changeStateAndNotify(newState: GuiSettingsState) {
    const oldState = this._state;
    this._state = newState;

    this.store();

    if (this.onChange) {
      this.onChange({ ...newState }, oldState);
    }
  }
}
