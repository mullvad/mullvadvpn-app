import { RoutePath } from '../../../shared/routes';

export interface SelectorItem<T> {
  label: string;
  value: T;
  disabled?: boolean;
  'data-testid'?: string;
  details?: { path: RoutePath; ariaLabel: string };
  subLabel?: string;
}
