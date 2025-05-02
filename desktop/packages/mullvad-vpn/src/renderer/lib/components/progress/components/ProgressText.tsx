import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressTextProps<T extends React.ElementType = 'span'> = LabelTinyProps<T>;

export const ProgressText = <T extends React.ElementType = 'span'>(props: ProgressTextProps<T>) => {
  const { disabled } = useProgress();
  return <LabelTiny color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'} {...props} />;
};
