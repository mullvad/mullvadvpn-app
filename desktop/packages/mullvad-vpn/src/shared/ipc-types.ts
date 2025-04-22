import { Action, Location } from 'history';

import { ITransitionSpecification } from '../renderer/lib/history';

export interface ICurrentAppVersionInfo {
  gui: string;
  daemon?: string;
  isConsistent: boolean;
  isBeta: boolean;
}

export interface IWindowShapeParameters {
  arrowPosition?: number;
}

export type SuppressOutdatedVersionOption = {
  type: 'suppress-outdated-version-warning';
};

export type LocationStateOptions = SuppressOutdatedVersionOption;

export type IChangelog = Array<string>;

export interface LocationState {
  scrollPosition: [number, number];
  expandedSections: Record<string, boolean>;
  transition: ITransitionSpecification;
  options?: LocationStateOptions[];
}

export interface IHistoryObject {
  entries: Location<LocationState>[];
  index: number;
  lastAction: Action;
}

export type ScrollPositions = Record<string, [number, number]>;
