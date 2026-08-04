import { Action, Location } from 'history';

import { type LocationType } from '../renderer/features/locations/types';
import { type TransitionType } from '../renderer/lib/history';

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

export type ScrollToAnchorId =
  | 'daita-enable-setting'
  | 'multihop-setting'
  | 'custom-dns-settings'
  | 'allow-lan-setting'
  | 'lockdown-mode-setting'
  | 'dns-blocker-setting'
  | 'mtu-setting'
  | 'obfuscation-setting'
  | 'port-setting'
  | 'mss-fix-setting'
  | 'quantum-resistant-setting';

export type ScrollToAnchorOption = {
  type: 'scroll-to-anchor';
  id: ScrollToAnchorId;
};

export type FilterViewOptions = {
  type: 'filter-view-options';
  locationType: LocationType;
};

export type LocationStateOptions =
  | SuppressOutdatedVersionOption
  | ScrollToAnchorOption
  | FilterViewOptions;

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
