import { BodySmall, type BodySmallProps } from '../../../text';
import { useMenuOptionContext } from '../../MenuOptionContext';

export type MenuOptionItemLabelProps = BodySmallProps;

export function MenuOptionItemLabel({ children, ...props }: MenuOptionItemLabelProps) {
  const { disabled } = useMenuOptionContext();

  return (
    <BodySmall color={disabled ? 'whiteAlpha20' : 'white'} {...props}>
      {children}
    </BodySmall>
  );
}
