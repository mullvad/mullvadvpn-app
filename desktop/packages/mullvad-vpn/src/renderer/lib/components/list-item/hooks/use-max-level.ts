import { levels } from '../levels';

export function useMaxLevel(level: number) {
  const maxLevel = Object.keys(levels).length - 1;
  return Math.min(level, maxLevel) as keyof typeof levels;
}
