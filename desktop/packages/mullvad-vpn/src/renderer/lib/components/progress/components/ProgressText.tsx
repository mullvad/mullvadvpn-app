import { Colors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressTextProps = LabelTinyProps;

export const ProgressText = ({ children, ...props }: ProgressTextProps) => {
  const { disabled } = useProgress();
  return (
    <LabelTiny color={disabled ? Colors.white40 : Colors.white} {...props}>
      {children}
    </LabelTiny>
  );
};
