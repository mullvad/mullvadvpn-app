import { LabelTinySemiBold, LabelTinySemiBoldProps } from '../../text';
import { useProgress } from '../ProgressContext';

export type ProgressTextProps<T extends React.ElementType = 'span'> = LabelTinySemiBoldProps<T>;

export const ProgressText = <T extends React.ElementType = 'span'>(props: ProgressTextProps<T>) => {
  const { disabled } = useProgress();
  return <LabelTinySemiBold color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'} {...props} />;
};
