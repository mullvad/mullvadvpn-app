import type { ValueOfArray } from '../utility-types';

export const translations = {
  allowedTags: ['b', 'br', 'em', 'a'],
  allowedVoidTags: ['br'],
} as const;

export type AllowedTag = ValueOfArray<typeof translations.allowedTags>;
export type AllowedVoidTag = ValueOfArray<typeof translations.allowedVoidTags>;
