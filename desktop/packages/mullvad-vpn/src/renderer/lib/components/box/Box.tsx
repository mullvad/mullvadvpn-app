import styled from 'styled-components';

import { Layout, LayoutProps } from '../layout';

interface BoxProps extends LayoutProps {
  $width?: string;
  $height?: string;
  center?: boolean;
}

const StyledBox = styled(Layout)<BoxProps>((props) => ({
  display: 'block',
  boxSizing: 'border-box',
  height: props.$height,
  minHeight: props.$height,
  width: props.$width,
  minWidth: props.$width,
}));

const StyledCenter = styled.div({
  display: 'grid',
  placeItems: 'center',
  height: '100%',
  width: '100%',
});

export const Box = ({ center, children, ...props }: React.PropsWithChildren<BoxProps>) => {
  const content = center ? <StyledCenter>{children}</StyledCenter> : children;
  return <StyledBox {...props}>{content}</StyledBox>;
};
