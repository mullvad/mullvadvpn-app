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

export type IChangelog = Array<string>;

export interface LocationState {
  scrollPosition: [number, number];
  expandedSections: Record<string, boolean>;
  transition: ITransitionSpecification;
}

export interface IHistoryObject {
  entries: Location<LocationState>[];
  index: number;
  lastAction: Action;
}

export type ScrollPositions = Record<string, [number, number]>;
