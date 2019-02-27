import { app } from 'electron';
import log from 'electron-log';
import * as fs from 'fs';
import * as path from 'path';

import { IGuiSettingsState } from '../shared/gui-settings-state';

export default class GuiSettings {
  get state(): IGuiSettingsState {
    return this.stateValue;
  }

  set autoConnect(newValue: boolean) {
    this.changeStateAndNotify({ ...this.stateValue, autoConnect: newValue });
  }

  get autoConnect(): boolean {
    return this.stateValue.autoConnect;
  }

  set monochromaticIcon(newValue: boolean) {
    this.changeStateAndNotify({ ...this.stateValue, monochromaticIcon: newValue });
  }

  get monochromaticIcon(): boolean {
    return this.stateValue.monochromaticIcon;
  }

  set startMinimized(newValue: boolean) {
    this.changeStateAndNotify({ ...this.stateValue, startMinimized: newValue });
  }

  get startMinimized(): boolean {
    return this.stateValue.startMinimized;
  }

  public onChange?: (newState: IGuiSettingsState, oldState: IGuiSettingsState) => void;

  private stateValue: IGuiSettingsState = {
    autoConnect: true,
    monochromaticIcon: false,
    startMinimized: false,
  };

  public load() {
    try {
      const settingsFile = this.filePath();
      const contents = fs.readFileSync(settingsFile, 'utf8');
      const settings = JSON.parse(contents);

      this.stateValue.autoConnect =
        typeof settings.autoConnect === 'boolean' ? settings.autoConnect : true;
      this.stateValue.monochromaticIcon = settings.monochromaticIcon || false;
      this.stateValue.startMinimized = settings.startMinimized || false;
    } catch (error) {
      log.error(`Failed to read GUI settings file: ${error}`);
    }
  }

  public store() {
    try {
      const settingsFile = this.filePath();

      fs.writeFileSync(settingsFile, JSON.stringify(this.stateValue));
    } catch (error) {
      log.error(`Failed to write GUI settings file: ${error}`);
    }
  }

  private filePath() {
    return path.join(app.getPath('userData'), 'gui_settings.json');
  }

  private changeStateAndNotify(newState: IGuiSettingsState) {
    const oldState = this.stateValue;
    this.stateValue = newState;

    this.store();

    if (this.onChange) {
      this.onChange({ ...newState }, oldState);
    }
  }
}
