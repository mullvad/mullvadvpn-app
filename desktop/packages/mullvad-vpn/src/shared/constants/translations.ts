import type { ValueOfArray } from '../utility-types';

export const translations = {
  allowedTags: ['b', 'br', 'em', 'a'],
} as const;

export type AllowedTag = ValueOfArray<typeof translations.allowedTags>;
