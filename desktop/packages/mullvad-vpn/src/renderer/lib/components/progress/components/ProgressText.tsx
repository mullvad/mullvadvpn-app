import { DeprecatedColors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressTextProps<T extends React.ElementType = 'span'> = LabelTinyProps<T>;

export const ProgressText = <T extends React.ElementType = 'span'>(props: ProgressTextProps<T>) => {
  const { disabled } = useProgress();
  return (
    <LabelTiny color={disabled ? DeprecatedColors.white40 : DeprecatedColors.white60} {...props} />
  );
};
