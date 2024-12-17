import styled from 'styled-components';

interface CenterProps {
  $width?: string;
  $height?: string;
}

export const Center = styled.div<CenterProps>((props) => ({
  display: 'grid',
  placeItems: 'center',
  height: props.$height,
  width: props.$width,
}));
