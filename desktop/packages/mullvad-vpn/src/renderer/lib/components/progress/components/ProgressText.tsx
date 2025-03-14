import { Colors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressTextProps<T extends React.ElementType = 'span'> = LabelTinyProps<T>;

export const ProgressText = <T extends React.ElementType = 'span'>(props: ProgressTextProps<T>) => {
  const { disabled } = useProgress();
  return <LabelTiny color={disabled ? Colors.white40 : Colors.white60} {...props} />;
};
