import { Icon, type IconProps } from '../../../icon';

export type NoteIconProps = IconProps;

export function NoteIcon(props: NoteIconProps) {
  return <Icon size="small" color="whiteAlpha60" {...props} />;
}
