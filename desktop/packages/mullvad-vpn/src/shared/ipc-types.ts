import { Action, Location } from 'history';

import { TransitionType } from '../renderer/lib/history';

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

export type ScrollToAnchorOption = {
  type: 'scroll-to-anchor';
  id: string;
};

export type LocationStateOptions = SuppressOutdatedVersionOption | ScrollToAnchorOption;

export type IChangelog = Array<string>;

export interface LocationState {
  scrollPosition: [number, number];
  expandedSections: Record<string, boolean>;
  transition: TransitionType;
  options?: LocationStateOptions[];
}

export interface IHistoryObject {
  entries: Location<LocationState>[];
  index: number;
  lastAction: Action;
}

export type ScrollPositions = Record<string, [number, number]>;

export type DaemonStatus = 'start-requested' | 'running' | 'stopped';
