import { Action, Location } from 'history';

export interface ICurrentAppVersionInfo {
  gui: string;
  daemon?: string;
  isConsistent: boolean;
  isBeta: boolean;
}

export interface IWindowShapeParameters {
  arrowPosition?: number;
}

export type IChangelog = Array<string>;

export interface IHistoryObject {
  entries: Location<unknown>[];
  index: number;
  lastAction: Action;
}

export type ScrollPositions = Record<string, [number, number]>;
